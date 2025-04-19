import * as anchor from '@project-serum/anchor';
import { PublicKey } from '@solana/web3.js';

export interface GlobalPool {
    superAdmin: PublicKey,      // 32
    loyaltyWallet: PublicKey,   // 8
    loyaltyFee: anchor.BN,      // 8
    totalRound: anchor.BN,      // 8
    recentPlays: GameData[]
}

export interface AccountData {
    name: String,
    nftMint: PublicKey,
}

export interface GameData {
    playTime: anchor.BN,        // 8
    rewardAmount: anchor.BN,    // 8
    token: anchor.BN,            // 8
}

export interface PlayerPool {
    player: PublicKey,              // 32
    round: anchor.BN,               // 8
    gameData: GameData,             // 24
    winTimes: anchor.BN,            // 8
    receivedReward: anchor.BN,      // 8
    claimableReward: anchor.BN      // 8
    claimableToken: anchor.BN[]      // 8 * 9
}

export interface ClimableInfo {
    user: string,
    amount: number
}
