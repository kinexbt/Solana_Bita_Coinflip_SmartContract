use anchor_lang::{prelude::*, AnchorDeserialize};
use anchor_spl::token::{Mint, Token, TokenAccount};

use solana_program::{pubkey::Pubkey, sysvar};

pub mod account;
pub mod constants;
pub mod error;
pub mod utils;

use account::*;
use constants::*;
use error::*;
use utils::*;

declare_id!("8QvsmhwMtFXYZn2LZzaUgSenBuHfoEeAtJcr1qtRr3wj");

#[program]
pub mod coinflip {
    use super::*;
    use solana_program::native_token::LAMPORTS_PER_SOL;
    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;

        sol_transfer_user(
            ctx.accounts.admin.to_account_info().clone(),
            ctx.accounts.reward_vault.to_account_info().clone(),
            ctx.accounts.system_program.to_account_info().clone(),
            ctx.accounts.rent.minimum_balance(0),
        )?;

        global_authority.super_admin = ctx.accounts.admin.key();
        global_authority.loyalty_wallet = LOYALTY_WALLET.parse::<Pubkey>().unwrap();
        global_authority.loyalty_fee = LOYALTY_FEE;

        Ok(())
    }

    pub fn initialize_player_pool(ctx: Context<InitializePlayerPool>) -> Result<()> {
        let player_pool = &mut ctx.accounts.player_pool;
        player_pool.player = ctx.accounts.owner.key();
        msg!("Owner: {:?}", player_pool.player.to_string());

        Ok(())
    }

    pub fn update(ctx: Context<Update>, loyalty_fee: u64) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;

        require!(
            ctx.accounts.admin.key() == global_authority.super_admin,
            GameError::InvalidAdmin
        );

        global_authority.loyalty_wallet = ctx.accounts.loyalty_wallet.key();
        global_authority.loyalty_fee = loyalty_fee;
        Ok(())
    }

    pub fn resize_global_pool(ctx: Context<ResizeUserPool>) -> Result<()> {
        let global_pool = &mut ctx.accounts.player_pool;

        //  resize userPool if needed
        let data_len = global_pool.data_len();
        
        if data_len == 332 {
            msg!("resizing account 332 to 656");
            resize_account(
                global_pool.clone(),
                656,
                ctx.accounts.owner.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            )?;
        }

        Ok(())
    }

    pub fn resize_user_pool(ctx: Context<ResizeUserPool>) -> Result<()> {
        let player_pool = &mut ctx.accounts.player_pool;

        //  resize userPool if needed
        let data_len = player_pool.to_account_info().data_len();
        
        if data_len == 96 {
            msg!("resizing account 96 to 168");
            resize_account(
                player_pool.to_account_info().clone(),
                168,
                ctx.accounts.owner.to_account_info().clone(),
                ctx.accounts.system_program.to_account_info().clone(),
            )?;
        }

        Ok(())
    }

    /**
        @disc: Main function to flip coin.
        @param:
            head_or_tail: indicate whether the player bet on head or tail       0: Tail, 1: Head
            bet_amount:    The SOL amount to deposit
    */
    pub fn play_game(ctx: Context<PlayGame>, head_or_tail: u64, bet_amount: u64) -> Result<()> {
        let player_pool = &mut ctx.accounts.player_pool;
        let global_authority = &mut ctx.accounts.global_authority;

        msg!(
            "Bet amount: {}
            Vault:{}
            Lamports: {}
            Owner Lamports: {}",
            bet_amount,
            ctx.accounts.reward_vault.to_account_info().key(),
            ctx.accounts.reward_vault.to_account_info().lamports(),
            ctx.accounts.owner.to_account_info().lamports()
        );

        require!(
            ctx.accounts.owner.to_account_info().lamports() > bet_amount,
            GameError::InsufficientUserBalance
        );

        require!(
            ctx.accounts.reward_vault.to_account_info().lamports() > 2 * bet_amount,
            GameError::InsufficientRewardVault
        );

        require!(
            ctx.accounts.loyalty_wallet.to_account_info().key() == global_authority.loyalty_wallet,
            GameError::InvalidLoyaltyWallet
        );

        require!(
            BET_SOL_AMOUNT.contains(&bet_amount),
            GameError::InvalidBetAmount
        );

        // charge extra 4% of bet_amount is fee
        let fee_price = bet_amount * global_authority.loyalty_fee / PERMILLE;

        // Transfer bet_amount Sol to this PDA
        sol_transfer_user(
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.reward_vault.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            bet_amount,
        )?;

        // Transfer SOL to the loyalty_wallet
        sol_transfer_user(
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.loyalty_wallet.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            fee_price,
        )?;

        // Generate random number
        let mut reward: u64 = 0;
        let clock = Clock::get()?;
        let timestamp = clock.unix_timestamp;

        let rand = get_rand(timestamp as u64, clock.slot) % 10000;

        // Compare random number and head_or_tail
        // 50% tail, 50% head
        if rand % 2 == head_or_tail {
            reward = 2 * bet_amount + fee_price;
        }

        msg!("Head or tail: {}, rand:{}, Reward: {}, token: SOL, Version: {}", head_or_tail, rand, reward, CF_VERSION);

        // Add game data to the blockchain
        player_pool.add_game_data(timestamp, reward, 0);
        global_authority.add_recent_play(timestamp, reward, 0, ctx.accounts.owner.key());

        global_authority.total_round += 1;

        Ok(())
    }

    /**
     * @disc: flip coin with token
     */
    pub fn play_token(
        ctx: Context<PlayToken>,
        head_or_tail: u64,
        bet_amount: u64,
        token_idx: u8,
    ) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;

        let vault_token_amount = ctx.accounts.ata_vault.amount;
        let user_token_amount = ctx.accounts.ata_user.amount;

        msg!(
            "Bet amount: {}
            Vault address:{}
            Vault tokens: {}
            User tokens: {}",
            bet_amount,
            ctx.accounts.ata_vault.to_account_info().key(),
            vault_token_amount,
            user_token_amount
        );

        require!(
            user_token_amount > bet_amount,
            GameError::InsufficientUserBalance
        );

        require!(
            vault_token_amount > 2 * bet_amount,
            GameError::InsufficientRewardVault
        );

        require!(
            ctx.accounts.loyalty_wallet.to_account_info().key() == global_authority.loyalty_wallet,
            GameError::InvalidLoyaltyWallet
        );

        require!(
            TOKEN_INFO[token_idx as usize].bet_amount.contains(&bet_amount),
            GameError::InvalidBetAmount
        );

        require!(
            TOKEN_INFO[token_idx as usize].mint == ctx.accounts.token_mint.key().to_string(),
            GameError::InvalidTokenMintAddress
        );

        // charge extra 4% of bet_amount is fee
        let fee_price = bet_amount * global_authority.loyalty_fee / PERMILLE;

        // Transfer bet_amount Sol to this PDA
        token_transfer_user(
            ctx.accounts.ata_user.to_account_info(),
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.ata_vault.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            bet_amount,
        )?;

        // Transfer SOL to the loyalty_wallet
        token_transfer_user(
            ctx.accounts.ata_user.to_account_info(),
            ctx.accounts.owner.to_account_info(),
            ctx.accounts.ata_loyalty.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            fee_price,
        )?;

        // Generate random number
        let mut reward: u64 = 0;
        let clock = Clock::get()?;
        let timestamp = clock.unix_timestamp;

        let rand = get_rand(timestamp as u64, clock.slot) % 10000;

        // Compare random number and head_or_tail
        // 50% tail, 50% head
        if rand % 2 == head_or_tail {
            reward = 2 * bet_amount + fee_price;
        }

        msg!("Head or tail: {}, rand:{}, Reward: {}, token: {}, Version: {}", head_or_tail, rand, reward, TOKEN_INFO[token_idx as usize].name, CF_VERSION);

        // Add game data to the blockchain
        let player_pool = &mut ctx.accounts.player_pool;
        player_pool.add_game_data(timestamp, reward, token_idx as u64 + 1);
        global_authority.add_recent_play(timestamp, reward, token_idx as u64 + 1, ctx.accounts.owner.key());

        global_authority.total_round += 1;

        Ok(())
    }

    /**
    The claim Reward function after playing
    */
    pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
        // let instruction_sysvar_account = ctx.accounts.instruction_sysvar_account.to_account_info();
        // let index = load_current_index_checked(&instruction_sysvar_account).unwrap();
        // require!(index == 0, GameError::InvalidClaim);

        let _vault_bump = ctx.bumps.vault;

        let player_pool = &mut ctx.accounts.player_pool;
        let reward = player_pool.claimable_reward;

        require!(
            ctx.accounts.vault.to_account_info().lamports() > reward,
            GameError::InsufficientRewardVault
        );

        msg!(
            "Withdrawer: {}
            Asking: {}
            Available: {}",
            ctx.accounts.owner.to_account_info().key(),
            player_pool.claimable_reward as u64 / LAMPORTS_PER_SOL,
            player_pool.game_data.reward_amount as u64 / LAMPORTS_PER_SOL
        );

        if reward > 0 {
            // Transfer SOL to the winner from the PDA
            sol_transfer_with_signer(
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.owner.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
                &[&[VAULT_AUTHORITY_SEED.as_bytes(), &[_vault_bump]]],
                reward,
            )?;
            player_pool.game_data.reward_amount = 0;
            player_pool.claimable_reward = 0;
        }
        Ok(())
    }

    /**
    The claim Reward function after playing
    */
    pub fn claim_token_reward(ctx: Context<ClaimTokenReward>, token_idx: u8) -> Result<()> {
        // let instruction_sysvar_account = ctx.accounts.instruction_sysvar_account.to_account_info();
        // let index = load_current_index_checked(&instruction_sysvar_account).unwrap();
        // require!(index == 0, GameError::InvalidClaim);

        require!(
            TOKEN_INFO[token_idx as usize].mint == ctx.accounts.token_mint.key().to_string(),
            GameError::InvalidTokenMintAddress
        );

        let player_pool = &mut ctx.accounts.player_pool;
        let reward = player_pool.claimable_token[token_idx as usize];
        // require!(
        //     ctx.accounts.ata_vault.to_account_info(). > reward,
        //     GameError::InsufficientRewardVault
        // );

        msg!(
            "Withdrawer: {}
            Asking: {}
            Available: {}
            Token_ids: {}",
            ctx.accounts.owner.to_account_info().key(),
            player_pool.claimable_reward as u64 / LAMPORTS_PER_SOL,
            player_pool.game_data.reward_amount as u64 / LAMPORTS_PER_SOL,
            token_idx
        );

        if reward > 0 {
            let _vault_bump = ctx.bumps.vault;

            token_transfer_with_signer(
                ctx.accounts.ata_vault.to_account_info(),
                ctx.accounts.vault.to_account_info(),
                ctx.accounts.ata_user.to_account_info(),
                ctx.accounts.token_program.to_account_info(),
                &[&[VAULT_AUTHORITY_SEED.as_bytes(), &[_vault_bump]]],
                reward,
            )?;
            
            player_pool.game_data.reward_amount = 0;
            player_pool.claimable_token[token_idx as usize] = 0;
        }
        Ok(())
    }

    /**
        @disc: Admin can withdraw SOL from the PDA
        @param:
            amount: The sol amount to withdraw from this PDA
    */
    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        let global_authority = &mut ctx.accounts.global_authority;
        require!(
            ctx.accounts.admin.key() == global_authority.super_admin
                || ctx.accounts.admin.key() == global_authority.loyalty_wallet,
            GameError::InvalidAdmin
        );

        let _vault_bump = ctx.bumps.reward_vault;

        sol_transfer_with_signer(
            ctx.accounts.reward_vault.to_account_info(),
            ctx.accounts.admin.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
            &[&[VAULT_AUTHORITY_SEED.as_bytes(), &[_vault_bump]]],
            amount,
        )?;

        let balance = ctx.accounts.ata_vault.amount;
        msg!("balance: {:?}", balance);
        
        token_transfer_with_signer(
            ctx.accounts.ata_vault.to_account_info(),
            ctx.accounts.reward_vault.to_account_info(),
            ctx.accounts.ata_admin.to_account_info(),
            ctx.accounts.token_program.to_account_info(),
            &[&[VAULT_AUTHORITY_SEED.as_bytes(), &[_vault_bump]]],
            balance,
        )?;

        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        init,
        space = 8 + GlobalPool::DATA_SIZE,
        seeds = [GLOBAL_AUTHORITY_SEED.as_bytes()],
        bump,
        payer = admin
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub reward_vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct InitializePlayerPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        init,
        space = 8 + PlayerPool::DATA_SIZE,
        seeds = [&owner.key().to_bytes(), PLAYER_POOL_SEED.as_bytes()],
        bump,
        payer = owner
    )]
    pub player_pool: Account<'info, PlayerPool>,

    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

#[derive(Accounts)]
pub struct Update<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    pub global_authority: Account<'info, GlobalPool>,

    #[account(mut)]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub loyalty_wallet: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ResizeUserPool<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    /// CHECK: change account size
    pub player_pool: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlayGame<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [&owner.key().to_bytes(), PLAYER_POOL_SEED.as_bytes()],
        bump
    )]
    pub player_pool: Account<'info, PlayerPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub reward_vault: AccountInfo<'info>,

    #[account(mut)]
    pub loyalty_wallet: SystemAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct PlayToken<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [&owner.key().to_bytes(), PLAYER_POOL_SEED.as_bytes()],
        bump
    )]
    pub player_pool: Account<'info, PlayerPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault: AccountInfo<'info>,
    
    pub token_mint: Account<'info, Mint>,

    //  user token account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = owner
    )]
    pub ata_user: Account<'info, TokenAccount>,
    
    #[account(
        mut, 
        token::mint = token_mint, 
        token::authority = vault,
    )]
    pub ata_vault: Box<Account<'info, TokenAccount>>,

    pub loyalty_wallet: SystemAccount<'info>,

    #[account(
        mut, 
        token::mint = token_mint, 
        token::authority = loyalty_wallet,
    )]
    pub ata_loyalty: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [&owner.key().to_bytes(), PLAYER_POOL_SEED.as_bytes()],
        bump
    )]
    pub player_pool: Account<'info, PlayerPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: instruction_sysvar_account cross checking
    #[account(address = sysvar::instructions::ID)]
    instruction_sysvar_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct ClaimTokenReward<'info> {
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        seeds = [&owner.key().to_bytes(), PLAYER_POOL_SEED.as_bytes()],
        bump
    )]
    pub player_pool: Account<'info, PlayerPool>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    pub global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub vault: AccountInfo<'info>,
    
    pub token_mint: Account<'info, Mint>,

    //  user token account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = owner
    )]
    pub ata_user: Account<'info, TokenAccount>,
    
    #[account(
        mut, 
        token::mint = token_mint, 
        token::authority = vault,
    )]
    pub ata_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,

    /// CHECK: instruction_sysvar_account cross checking
    #[account(address = sysvar::instructions::ID)]
    instruction_sysvar_account: AccountInfo<'info>,
}

#[derive(Accounts)]
pub struct Withdraw<'info> {
    #[account(mut)]
    admin: Signer<'info>,

    #[account(
        mut,
        seeds = [GLOBAL_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    global_authority: Box<Account<'info, GlobalPool>>,

    #[account(
        mut,
        seeds = [VAULT_AUTHORITY_SEED.as_bytes()],
        bump,
    )]
    /// CHECK: This is not dangerous because we don't read or write from this account
    pub reward_vault: AccountInfo<'info>,
    
    pub token_mint: Account<'info, Mint>,

    //  admin token account
    #[account(
        mut,
        token::mint = token_mint,
        token::authority = admin
    )]
    pub ata_admin: Account<'info, TokenAccount>,
    
    #[account(
        mut, 
        token::mint = token_mint, 
        token::authority = reward_vault,
    )]
    pub ata_vault: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,

    pub system_program: Program<'info, System>,
}
