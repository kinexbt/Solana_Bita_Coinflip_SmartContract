import { Program, web3 } from '@project-serum/anchor';
import * as anchor from '@project-serum/anchor';
import { Keypair, PublicKey, SystemProgram, SYSVAR_RENT_PUBKEY, PartiallyDecodedInstruction, Transaction, } from '@solana/web3.js';
import fs from 'fs';
import NodeWallet from '@project-serum/anchor/dist/cjs/nodewallet';
import { ClimableInfo, GameData, GlobalPool, PlayerPool } from './types';
import { IDL as GameIDL } from "../target/types/coinflip";
import { bs58 } from '@project-serum/anchor/dist/cjs/utils/bytes';
import { findProgramAddressSync } from '@project-serum/anchor/dist/cjs/utils/pubkey';
import { GLOBAL_AUTHORITY_SEED, LAMPORTS, NONCE, PLAYER_POOL_SEED, TOKEN_ADDR, USER_POOL_SIZE, VAULT_AUTHORITY_SEED, getAssociatedTokenAccount, network, programId } from './config';
import { TOKEN_PROGRAM_ID } from "@solana/spl-token";

// Set the initial program and provider
let program: Program = null;
let provider: anchor.Provider = null;

anchor.setProvider(anchor.AnchorProvider.local(web3.clusterApiUrl(network)));
provider = anchor.getProvider();

let solConnection = anchor.getProvider().connection;
let payer: NodeWallet = null;

export const setClusterConfig = async (cluster: web3.Cluster, keypair: string, rpc?: string) => {
    if (!rpc) {
        solConnection = new web3.Connection(web3.clusterApiUrl(cluster));
    } else {
        solConnection = new web3.Connection(rpc);
    }

    //console.log("Connection", solConnection);
    const walletKeypair = Keypair.fromSecretKey(Uint8Array.from(JSON.parse(fs.readFileSync(keypair, 'utf-8'))), { skipValidation: true });
    payer = new NodeWallet(walletKeypair);

    // Configure the client to use the local cluster.
    anchor.setProvider(new anchor.AnchorProvider(solConnection, payer, { skipPreflight: true, commitment: 'confirmed' }));
    provider = anchor.getProvider();
    console.log('Wallet Address: ', payer.publicKey.toBase58());

    // Generate the program client from IDL.
    program = new anchor.Program(GameIDL as anchor.Idl, programId);
    console.log('ProgramId: ', program.programId.toBase58());
}

export const initProject = async () => {
    const [globalAuthority, bump] = findProgramAddressSync([Buffer.from(GLOBAL_AUTHORITY_SEED)], program.programId);
    console.log("global pool: ", globalAuthority.toBase58());

    const [rewardVault, vaultBump] = findProgramAddressSync([Buffer.from(VAULT_AUTHORITY_SEED)], program.programId);

    const tx = await program.methods
        .initialize()
        .accounts({
            admin: provider.publicKey,
            globalAuthority,
            rewardVault: rewardVault,
            systemProgram: SystemProgram.programId,
            rent: SYSVAR_RENT_PUBKEY
        })
        .transaction();

    const txId = await provider.sendAndConfirm(tx, [], {
        commitment: "confirmed",
    });

    console.log("txHash =", txId);

    return true;
}

export const update = async (loyaltyWallet: PublicKey, loyaltyFee: number) => {
    const tx = await updateTx(provider.publicKey, loyaltyWallet, loyaltyFee);

    const txId = await provider.sendAndConfirm(tx, [], {
        commitment: "confirmed",
    });

    console.log("txHash =", txId);
}

export const playGame = async (setValue: number, deposit: number) => {
    const tx = await createPlayGameTx(payer.publicKey, setValue, deposit);

    try {
        const txId = await provider.sendAndConfirm(tx, [], {
            commitment: "confirmed",
        });
        console.log("txHash =", txId);
    } catch (e) {
        console.log(e)
    }

}


export const playToken = async (setValue: number, deposit: number, token: number) => {
    const tx = await createPlayTokenTx(payer.publicKey, setValue, deposit, token);

    try {
        const txId = await provider.sendAndConfirm(tx, [], {
            commitment: "confirmed",
        });
        console.log("txHash =", txId);
    } catch (e) {
        console.log(e)
    }

}

export const claim = async () => {
    const tx = await createClaimTx(payer.publicKey);

    try {
        const txId = await provider.sendAndConfirm(tx, [], {
            commitment: "confirmed",
        });

        console.log("txHash =", txId);
    } catch (e) {
        console.log(e);
    }
}

export const claimToken = async (token_idx: number) => {
    const tx = await createClaimTokenTx(payer.publicKey, token_idx);

    try {
        const txId = await provider.sendAndConfirm(tx, [], {
            commitment: "confirmed",
        });

        console.log("txHash =", txId);
    } catch (e) {
        console.log(e);
    }
}

export const withdraw = async (amount: number) => {
    const tx = await createWithDrawTx(provider.publicKey, amount);
    const txId = await provider.sendAndConfirm(tx, [], {
        commitment: "confirmed",
    });

    console.log("txHash =", txId);
}

//  Functions for making transactions
export const initUserPoolTx = async (userAddress: PublicKey,) => {
    const [userPool, bump] = PublicKey.findProgramAddressSync(
        [userAddress.toBuffer(), Buffer.from(PLAYER_POOL_SEED)],
        program.programId);

    const tx = await program.methods
        .initializePlayerPool()
        .accounts({
            owner: userAddress, playerPool: userPool,
        })
        .transaction();

    return tx;
}

export const updateTx = async (userAddress: PublicKey, loyaltyWallet: PublicKey, loyaltyFee: number) => {
    const [globalAuthority, bump] = findProgramAddressSync([Buffer.from(GLOBAL_AUTHORITY_SEED)], program.programId);

    const tx = await program.methods
        .update(new anchor.BN(loyaltyFee * 10))
        .accounts({
            admin: userAddress,
            globalAuthority, loyaltyWallet
        })
        .transaction();

    return tx;
}

export const createPlayGameTx = async (userAddress: PublicKey, setNum: number, deposit: number) => {
    const [globalAuthority, bump] = findProgramAddressSync([Buffer.from(GLOBAL_AUTHORITY_SEED)], program.programId);
    console.log("Global Pool: ", globalAuthority.toBase58());

    const global = await getGlobalState();
    if (global === null) {
        console.log("failed to get global state ;(");
        return;
    }

    const [rewardVault, vaultBump] = findProgramAddressSync([Buffer.from(VAULT_AUTHORITY_SEED)], program.programId);
    console.log("Reward Vault: ", rewardVault.toBase58());

    const [userPool, userBump] = PublicKey.findProgramAddressSync(
        [userAddress.toBuffer(), Buffer.from(PLAYER_POOL_SEED)],
        program.programId);
    console.log("Player Pool: ", userPool.toBase58());

    let tx = new Transaction();

    let poolAccount = await solConnection.getAccountInfo(userPool);
    if (poolAccount === null || poolAccount.data === null) {
        console.log('init User Pool');
        tx.add(await initUserPoolTx(userAddress));
    }

    const playTx = await program.methods
        .playGame(new anchor.BN(setNum), new anchor.BN(deposit * LAMPORTS))
        .accounts({
            owner: userAddress,
            playerPool: userPool,
            globalAuthority,
            rewardVault: rewardVault,
            loyaltyWallet: global.loyaltyWallet,
            systemProgram: SystemProgram.programId
        })
        .transaction();

    tx.add(playTx);

    return tx;
}

export const createPlayTokenTx = async (userAddress: PublicKey, setNum: number, deposit: number, token_idx: number) => {
    const [globalAuthority, bump] = findProgramAddressSync([Buffer.from(GLOBAL_AUTHORITY_SEED)], program.programId);
    console.log("Global Pool: ", globalAuthority.toBase58());

    const global = await getGlobalState();
    if (global === null) {
        console.log("failed to get global state ;(");
        return;
    }

    const [vault, vaultBump] = findProgramAddressSync([Buffer.from(VAULT_AUTHORITY_SEED)], program.programId);
    console.log("Reward Vault: ", vault.toBase58());

    const [userPool, userBump] = PublicKey.findProgramAddressSync(
        [userAddress.toBuffer(), Buffer.from(PLAYER_POOL_SEED)],
        program.programId);
    console.log("Player Pool: ", userPool.toBase58());

    console.log("Token mint: ", TOKEN_ADDR.toBase58());

    const ataUser = getAssociatedTokenAccount(userAddress, TOKEN_ADDR);
    console.log("ATA User: ", ataUser.toBase58());
    const ataVault = getAssociatedTokenAccount(vault, TOKEN_ADDR);
    console.log("ATA Vault: ", ataVault.toBase58());
    const ataLoyalty = getAssociatedTokenAccount(global.loyaltyWallet, TOKEN_ADDR);
    console.log("Loyalty: ", global.loyaltyWallet.toBase58());
    console.log("ATA Loyalty: ", ataLoyalty.toBase58());

    let tx = new Transaction();

    let poolAccount = await solConnection.getAccountInfo(userPool);
    if (poolAccount === null || poolAccount.data === null) {
        console.log('init User Pool');
        tx.add(await initUserPoolTx(userAddress));
    }

    const playTx = await program.methods
        .playToken(new anchor.BN(setNum), new anchor.BN(5_000_000_000), token_idx)
        .accounts({
            owner: userAddress,
            playerPool: userPool,
            globalAuthority,
            vault,
            tokenMint: TOKEN_ADDR,
            ataUser,
            ataVault,
            loyaltyWallet: global.loyaltyWallet,
            ataLoyalty,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
        })
        .transaction();

    tx.add(playTx);

    return tx;
}

export const createClaimTx = async (userAddress: PublicKey) => {

    const [globalAuthority, bump] = findProgramAddressSync([Buffer.from(GLOBAL_AUTHORITY_SEED)], program.programId);
    console.log('Global Pool: ', globalAuthority.toBase58());

    const [vault, vaultBump] = findProgramAddressSync([Buffer.from(VAULT_AUTHORITY_SEED)], program.programId);
    console.log("vault: ", vault.toBase58());

    const [userPool, userBump] = PublicKey.findProgramAddressSync(
        [userAddress.toBuffer(), Buffer.from(PLAYER_POOL_SEED)],
        program.programId);
    console.log("Player Pool: ", userPool.toBase58());

    const tx = await program.methods
        .claimReward()
        .accounts({
            owner: userAddress,
            playerPool: userPool,
            globalAuthority,
            vault,
            systemProgram: SystemProgram.programId,
            instructionSysvarAccount: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY
        })
        .transaction();

    return tx;
}

export const createClaimTokenTx = async (userAddress: PublicKey, token_idx: number) => {

    const [globalAuthority, bump] = findProgramAddressSync([Buffer.from(GLOBAL_AUTHORITY_SEED)], program.programId);
    console.log('Global Pool: ', globalAuthority.toBase58());

    const [vault, vaultBump] = findProgramAddressSync([Buffer.from(VAULT_AUTHORITY_SEED)], program.programId);

    const [userPool, userBump] = PublicKey.findProgramAddressSync(
        [userAddress.toBuffer(), Buffer.from(PLAYER_POOL_SEED)],
        program.programId);
    console.log("Player Pool: ", userPool.toBase58());

    console.log("Token mint: ", TOKEN_ADDR.toBase58());
    const ataUser = getAssociatedTokenAccount(userAddress, TOKEN_ADDR);
    console.log("ATA User: ", ataUser.toBase58());
    const ataVault = getAssociatedTokenAccount(vault, TOKEN_ADDR);
    console.log("ATA Vault: ", ataVault.toBase58());

    const tx = await program.methods
        .claimTokenReward(token_idx)
        .accounts({
            owner: userAddress,
            playerPool: userPool,
            globalAuthority,
            vault,
            tokenMint: TOKEN_ADDR,
            ataUser,
            ataVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            instructionSysvarAccount: anchor.web3.SYSVAR_INSTRUCTIONS_PUBKEY
        })
        .transaction();

    return tx;
}

export const createWithDrawTx = async (userAddress: PublicKey, deposit: number) => {

    const [globalAuthority, bump] = findProgramAddressSync([Buffer.from(GLOBAL_AUTHORITY_SEED)], program.programId);
    console.log('Global Pool: ', globalAuthority.toBase58());

    const [rewardVault, vaultBump] = findProgramAddressSync([Buffer.from(VAULT_AUTHORITY_SEED)], program.programId);

    console.log("Token mint: ", TOKEN_ADDR.toBase58());
    const ataAdmin = getAssociatedTokenAccount(userAddress, TOKEN_ADDR);
    console.log("ATA User: ", ataAdmin.toBase58());
    const ataVault = getAssociatedTokenAccount(rewardVault, TOKEN_ADDR);
    console.log("ATA Vault: ", ataVault.toBase58());

    const tx = await program.methods
        .withdraw(new anchor.BN(deposit * LAMPORTS))
        .accounts({
            admin: userAddress,
            globalAuthority,
            rewardVault,
            tokenMint: TOKEN_ADDR,
            ataAdmin,
            ataVault,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId
        })
        .transaction();

    return tx;
}

export const getGlobalState = async (): Promise<GlobalPool | null> => {
    const [globalAuthority, bump] = findProgramAddressSync([Buffer.from(GLOBAL_AUTHORITY_SEED)], program.programId);

    console.log("Global Pool: ", globalAuthority.toBase58());

    try {
        return await program.account.globalPool.fetch(globalAuthority, "confirmed") as unknown as GlobalPool;
    }
    catch
    {
        return null;
    }
}

export const getGlobalInfo = async () => {
    const globalPool = await getGlobalState();

    console.log(globalPool);

    // return {
    //     admin: globalPool.superAdmin.toBase58(),
    //     fee: globalPool.loyaltyFee.toNumber(),
    //     totalRound: globalPool.totalRound.toNumber(),
    //     recentPlays: globalPool.recentPlays.map((data: GameData, index) => {
    //         return `playTime: ${new Date(data.playTime.toNumber() * 1000).toLocaleString()} rewardAmount: ${data.rewardAmount.toNumber()} token: ${data.token.toNumber()}`;
    //     })
    // };
}

export const getUserPoolState = async (userAddress: PublicKey): Promise<PlayerPool | null> => {
    if (!userAddress) return null;

    const [userPool, userBump] = PublicKey.findProgramAddressSync(
        [userAddress.toBuffer(), Buffer.from(PLAYER_POOL_SEED)],
        program.programId);
    console.log('Player Pool: ', userPool.toBase58());

    try {
        return await program.account.playerPool.fetch(userPool, "confirmed") as unknown as PlayerPool;
    }
    catch
    {
        return null;
    }
}

export const getUserInfo = async (userAddr: PublicKey) => {
    const userPool = await getUserPoolState(userAddr);

    return {
        player: userPool.player.toBase58(),
        round: userPool.round.toNumber(),
        lastPlayTime: new Date(userPool.gameData.playTime.toNumber() * 1000).toLocaleString(),
        lastPlayReward: userPool.gameData.rewardAmount.toNumber(),
        lastPlayToken: userPool.gameData.token.toNumber(),
        winTimes: userPool.winTimes.toNumber(),
        receivedReward: userPool.receivedReward.toNumber(),
        claimableReward: userPool.claimableReward.toNumber(),
        claimableToken: userPool.claimableToken.map((amount, index) => {
            return `${index}: ${amount.toNumber()}`;
        })
    };
}

export const getClaimableInfo = async () => {

    let userPools = await solConnection.getProgramAccounts(
        programId,
        {
            filters: [{
                dataSize: USER_POOL_SIZE,
            }],
        }
    );

    let result: ClimableInfo[] = [];

    for (let idx = 0; idx < userPools.length; idx++) {
        const data = userPools[idx].account.data;

        const user = new PublicKey(data.subarray(8, 40));
        const buf = data.subarray(88, 96).reverse();
        const claimableAmount = new anchor.BN(buf);

        result.push({
            user: user.toBase58(),
            amount: claimableAmount.toNumber()
        })
    }

    return result;
}

// Get signautres related with Program Pubkey
export const getAllTransactions = async (programId: PublicKey) => {
    const data = await solConnection.getSignaturesForAddress(programId, {}, "confirmed");
    let result = [];
    console.log(`Tracked ${data.length} signature\nStart parsing Txs....`);
    let txdata = data.filter((tx) => tx.err === null);
    for (let i = 0; i < txdata.length; i++) {
        let rt = await getDataFromSignature(txdata[i].signature);
        if (rt !== undefined) {
            result.push(rt)
        }
    }
    return result;
}

// Parse activity from a transaction siganture
export const getDataFromSignature = async (sig: string) => {

    // Get transaction data from on-chain
    let tx;
    try {
        tx = await solConnection.getParsedTransaction(sig, "confirmed");
    }
    catch (e) {
    }

    if (!tx) {
        console.log(`Can't get Transaction for ${sig}`);
        return;
    }

    if (tx.meta?.err !== null) {
        console.log(`Failed Transaction: ${sig}`);
        return;
    }

    // Parse activty by analyze fetched Transaction data
    let length = tx.transaction.message.instructions.length;
    let valid = 0;
    let hash = "";
    let ixId = -1;
    for (let i = 0; i < length; i++) {
        hash = (tx.transaction.message.instructions[i] as PartiallyDecodedInstruction).data;
        if (hash !== undefined && hash.slice(0, 8) === NONCE) {
            valid = 1;
        }
        if (valid === 1) {
            ixId = i;
            break;
        }
    }

    if (ixId === -1 || valid === 0) {
        return;
    }

    let ts = tx.slot ?? 0;
    if (!tx.meta.innerInstructions) {
        console.log(`Can't parse innerInstructions ${sig}`);
        return;
    }


    let accountKeys = (tx.transaction.message.instructions[ixId] as PartiallyDecodedInstruction).accounts;
    let signer = accountKeys[0].toBase58();

    let bytes = bs58.decode(hash);
    let a = bytes.slice(8, 16).reverse();
    let type = new anchor.BN(a).toNumber();
    let b = bytes.slice(16, 24).reverse();
    let sol_price = new anchor.BN(b).toNumber();

    let state = type === ts % 2 ? 1 : 0;

    let result = {
        type: type, address: signer, bet_amount: sol_price, block_hash: ts, win: state, signature: sig,
    };

    return result;
};

export const resizeAllUserPool = async () => {

    let userPools = await solConnection.getProgramAccounts(
        programId,
        {
            filters: [{
                dataSize: USER_POOL_SIZE,
            }],
        }
    );

    for (let idx = 0; idx < userPools.length; idx++) {

        const tx = await program.methods
            .resizeUserPool()
            .accounts({
                owner: new PublicKey("G2sc5mU3eLRkbRupnupzB3NTzZ85bnc9L1ReAre9dzFU"),
                playerPool: userPools[idx].pubkey,
                systemProgram: SystemProgram.programId
            })
            .transaction();

        try {
            const txId = await provider.sendAndConfirm(tx, [], {
                commitment: "confirmed",
            });

            console.log("resized: ", userPools[idx].pubkey.toBase58());
            console.log("txHash =", txId);
        } catch (e) {
            console.log(e);
        }
    }
}

export const resizeGlobalPool = async () => {
    const tx = await program.methods
        .resizeGlobalPool()
        .accounts({
            owner: new PublicKey("G2sc5mU3eLRkbRupnupzB3NTzZ85bnc9L1ReAre9dzFU"),
            playerPool: new PublicKey("WEzV6NZKnNuxGLjSUjfczyThec13jx4bEcFG7TqSPHP"),
            systemProgram: SystemProgram.programId
        })
        .transaction();
    try {
        const txId = await provider.sendAndConfirm(tx, [], {
            commitment: "confirmed",
        });

        console.log("txHash =", txId);
    } catch (e) {
        console.log(e);
    }
}