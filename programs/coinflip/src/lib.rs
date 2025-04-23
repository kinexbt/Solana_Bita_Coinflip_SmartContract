use anchor_lang::{prelude::*, AnchorDeserialize};

use solana_program::pubkey::Pubkey;

pub mod account;
pub mod constants;
pub mod error;
pub mod utils;

use account::*;
use constants::*;
use error::*;
use utils::*;

declare_id!("2yGiLmgFhZvHvLYMshTwcAsZ9Ja2wNxxQiSWKhmPmqZe");

#[program]
pub mod coinflip {
    use super::*;
    use solana_program::native_token::LAMPORTS_PER_SOL;
    pub fn initialize(
        ctx: Context<Initialize>,
        operate_admin: Pubkey,
        financial_admin: Pubkey,
        update_admin: Pubkey,
    ) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;

        // sol_transfer_user(
        //     ctx.accounts.admin.to_account_info().clone(),
        //     ctx.accounts.casino_vault.to_account_info().clone(),
        //     ctx.accounts.system_program.to_account_info().clone(),
        //     ctx.accounts.rent.minimum_balance(0),
        // )?;

        global_authority.super_admin = ctx.accounts.admin.key();
        global_authority.operation_authority = operate_admin.key();
        global_authority.finance_authority = financial_admin.key();
        global_authority.update_authority = update_admin.key();
        global_authority.rtp = RTP;
        global_authority.max_win_amount = MAX_WIN_AMOUNT;
        global_authority.min_bet_amount = MIN_BET_AMOUNT;

        Ok(())
    }
    /**
        @disc: Main function to flip coin.
        @param:
            head_or_tail: indicate whether the player bet on head or tail       0: Tail, 1: Head
            bet_amount:    The SOL amount to deposit
    */
    pub fn play_game(ctx: Context<PlayGame>, is_head: bool, bet_amount: u64) -> Result<()> {
        let player_pool = &mut ctx.accounts.player_pool;
        let player = &ctx.accounts.owner;
        let global_authority = &ctx.accounts.global_authority;

        require!(
            global_authority.min_bet_amount < bet_amount,
            GameError::InvalidBetAmount
        );

        let bet_amount_f64 = bet_amount as f64;
        let rtp_f64 = global_authority.rtp as f64;
        let rtp_ratio = rtp_f64 / 100.0;
        let double_bet = bet_amount_f64 * 2.0;
        let potential_win = double_bet * rtp_ratio;
        let net_gain = potential_win - bet_amount_f64;
        let net_gain_u64 = net_gain as u64;
        let max_win_amount_u64 = global_authority.max_win_amount;

        require!(
            net_gain_u64 < max_win_amount_u64,
            GameError::InvalidBetAmountMaxWinAmountViolation
        );

        // require!(
        //     (bet_amount * 2 * (global_authority.rtp / 100) - bet_amount)
        //         < global_authority.max_win_amount,
        //     GameError::InvalidBetAmountMaxWinAmountViolation
        // );

        require!(
            ctx.accounts.owner.to_account_info().lamports() > bet_amount,
            GameError::InsufficientUserBalance
        );

        require!(
            ctx.accounts.casino_vault.to_account_info().lamports() > bet_amount,
            GameError::InsufficientCasinoVault
        );

        // Transfer rent fee for PDA of player pool
        sol_transfer_user(
            ctx.accounts.owner.to_account_info().clone(),
            player_pool.to_account_info().clone(),
            ctx.accounts.system_program.to_account_info().clone(),
            ctx.accounts.rent.minimum_balance(0),
        )?;

        // Transfer bet_amount Sol to this PDA from User Wallet
        sol_transfer_user(
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.game_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            bet_amount,
        )?;

        player_pool.status = GameStatus::Processing;
        player_pool.round = 1;
        player_pool.first_bet = bet_amount;
        player_pool.player = player.key();

        if is_head == true {
            msg!(
                "User's choice is Head, bet amount is {}SOL",
                bet_amount / LAMPORTS_PER_SOL
            );
        } else {
            msg!(
                "User's choice is Tail, bet amount is {}SOL",
                bet_amount / LAMPORTS_PER_SOL
            );
        }

        Ok(())
    }

    /**
    The setting result function to determine whether player Win or Lose
    */
    pub fn set_result(ctx: Context<SetResult>, round_id: u8, is_win: bool) -> Result<()> {
        let player_pool = &mut ctx.accounts.player_pool;
        let game_bump = ctx.bumps.game_vault;
        let casino_bump = ctx.bumps.casino_vault;
        let global_authority = &ctx.accounts.global_authority;
        let game_vault = &mut ctx.accounts.game_vault;
        let casino_vault = &mut ctx.accounts.casino_vault;
        let vault_balance = game_vault.lamports();
        let win_balance = vault_balance * 2 * global_authority.rtp / 100;

        msg!(
            "RoundId: {}, PlayerPoolRoundId: {}",
            round_id,
            player_pool.round
        );
        require!(round_id == player_pool.round, GameError::RoundNumMismatch);

        if is_win == true {
            // Transfer bet_amount Sol to this PDA from casino bank
            sol_transfer_with_signer(
                casino_vault.to_account_info(),
                game_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                &[&[VAULT_AUTHORITY_SEED.as_bytes(), &[casino_bump]]],
                win_balance - vault_balance,
            )?;

            player_pool.status = GameStatus::Win
        } else {
            sol_transfer_with_signer(
                game_vault.to_account_info(),
                casino_vault.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                &[&[
                    ctx.accounts.owner.key().as_ref(),
                    VAULT_AUTHORITY_SEED.as_bytes(),
                    &[game_bump],
                ]],
                vault_balance,
            )?;

            player_pool.status = GameStatus::Lose;

            // Here, add closePda function
            // **game_vault.to_account_info().try_borrow_mut_lamports()? = 0;
            // **player_pool.to_account_info().try_borrow_mut_lamports()? = 0;
        }

        Ok(())
    }

    /**
    Double Bet function when the user want to do that after win the game
    */
    pub fn double_bet(ctx: Context<DoubleBet>) -> Result<()> {
        let player_pool = &mut ctx.accounts.player_pool;
        let round = player_pool.round;
        let player = &ctx.accounts.owner;
        let game_vault = &mut ctx.accounts.game_vault;
        let global_authority = &ctx.accounts.global_authority;
        let game_balance = game_vault.lamports();

        require!(
            player_pool.status == GameStatus::Win,
            GameError::NotAllowedDoubleBet
        );

        require!(
            (game_balance * 2 * global_authority.rtp / 100 - player_pool.first_bet)
                < global_authority.max_win_amount,
            GameError::InvalidBetAmountMaxWinAmountViolation
        );

        require!(
            player.key() == player_pool.player,
            GameError::NotOriginalPlayer
        );

        player_pool.update_round(GameStatus::Processing, round + 1);

        //  = GameStatus::Processing;
        // msg!("Initial round num: {}", player_pool.round);
        // player_pool.round += 1;
        // msg!("Double Bet round num: {}", player_pool.round);

        Ok(())
    }

    /**
    The claim Reward function for User after playing and Win
    */
    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        let player_pool = &mut ctx.accounts.player_pool;
        let player = &ctx.accounts.player;
        let game_bump = ctx.bumps.game_vault;
        let game_vault = &mut ctx.accounts.game_vault;
        let game_balance = game_vault.lamports();

        require!(
            player_pool.status == GameStatus::Win,
            GameError::NotAllowedStatus
        );

        msg!(
            "Withdrawer: {}
            Amount: {}",
            player.key(),
            game_balance,
        );

        // Transfer SOL to the winner from the PDA
        sol_transfer_with_signer(
            game_vault.to_account_info(),
            ctx.accounts.player.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &[&[
                ctx.accounts.player.key().as_ref(),
                VAULT_AUTHORITY_SEED.as_bytes(),
                &[game_bump],
            ]],
            game_balance,
        )?;

        player_pool.status = GameStatus::Finished;

        // Add the closing PDA part
        // **game_vault.to_account_info().try_borrow_mut_lamports()? = 0;
        // **player_pool.to_account_info().try_borrow_mut_lamports()? = 0;

        Ok(())
    }

    /**
        @disc: Admin can withdraw SOL from the PDA
        @param:
            amount: The sol amount to withdraw from this PDA
    */
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let global_authority = &ctx.accounts.global_authority;
        let financial_authority = &ctx.accounts.financial_admin;
        let recipient = &ctx.accounts.recipient;
        let casino_bump = ctx.bumps.casino_vault;
        let casino_vault = &ctx.accounts.casino_vault;

        require!(
            financial_authority.key() == global_authority.finance_authority,
            GameError::UnauthorizedFinanceAdmin
        );

        require!(
            casino_vault.lamports() > amount,
            GameError::InsufficientCasinoVault
        );

        sol_transfer_with_signer(
            ctx.accounts.casino_vault.to_account_info(),
            recipient.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &[&[VAULT_AUTHORITY_SEED.as_bytes(), &[casino_bump]]],
            amount,
        )?;

        let balance = ctx.accounts.casino_vault.to_account_info().lamports();

        msg!("Remaining balance: {:?}", balance);

        Ok(())
    }

    pub fn set_rtp(ctx: Context<SetGlobalPool>, new_rtp: u64) -> Result<()> {
        require!(new_rtp < 100, GameError::InvalidRtp);

        ctx.accounts.global_pool.rtp = new_rtp;

        Ok(())
    }

    pub fn set_max_win_amount(ctx: Context<SetGlobalPool>, new_max_win_amount: u64) -> Result<()> {
        ctx.accounts.global_pool.max_win_amount = new_max_win_amount;
        Ok(())
    }

    pub fn set_min_bet_amount(ctx: Context<SetGlobalPool>, new_min_bet_amount: u64) -> Result<()> {
        ctx.accounts.global_pool.min_bet_amount = new_min_bet_amount;
        Ok(())
    }

    pub fn set_operation_authority(
        ctx: Context<SetAuthority>,
        new_operation_authority: Pubkey,
    ) -> Result<()> {
        ctx.accounts.global_pool.operation_authority = new_operation_authority;
        Ok(())
    }

    pub fn set_finance_authority(
        ctx: Context<SetAuthority>,
        new_finance_authority: Pubkey,
    ) -> Result<()> {
        ctx.accounts.global_pool.finance_authority = new_finance_authority;
        Ok(())
    }

    pub fn set_update_authority(
        ctx: Context<SetAuthority>,
        new_update_authority: Pubkey,
    ) -> Result<()> {
        ctx.accounts.global_pool.update_authority = new_update_authority;
        Ok(())
    }
}
