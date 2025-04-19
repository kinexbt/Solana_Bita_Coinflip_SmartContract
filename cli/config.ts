import { PublicKey } from '@solana/web3.js';
import { ASSOCIATED_TOKEN_PROGRAM_ID, TOKEN_PROGRAM_ID } from "@solana/spl-token";

export const LAMPORTS = 1000000000;
export const USER_POOL_SIZE = 168;

export const GLOBAL_AUTHORITY_SEED = "global-authority";
export const VAULT_AUTHORITY_SEED = "vault-authority";
export const PLAYER_POOL_SEED = "player-pool";
export const NONCE = "4QUPibxi";

export const network = "devnet";

export const programId = new PublicKey("B7zxxL7pyzsCojCzdpJx3CPLVC9n5MhDUZ2jxWMf5MRp");

// export const TOKEN_ADDR = new PublicKey("5NyADEEXaoniwHkwWYEVuuB9mAtnXSCsEfmDm4gFvhQM");
export const TOKEN_ADDR = new PublicKey("F6weWmuc1vwdL4u38Ro9jKXHEMjP9BoNdWMF5o5TvtJf");

export const getAssociatedTokenAccount = (
    ownerPubkey: PublicKey,
    mintPk: PublicKey
): PublicKey => {
    let associatedTokenAccountPubkey = (PublicKey.findProgramAddressSync(
        [
            ownerPubkey.toBytes(),
            TOKEN_PROGRAM_ID.toBytes(),
            mintPk.toBytes(), // mint address
        ],
        ASSOCIATED_TOKEN_PROGRAM_ID
    ))[0];

    return associatedTokenAccountPubkey;
}