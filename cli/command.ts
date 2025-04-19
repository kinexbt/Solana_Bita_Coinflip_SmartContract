import { program } from 'commander';
import {
    PublicKey,
} from '@solana/web3.js';
import { claim, claimToken, getClaimableInfo, getGlobalInfo, getUserInfo, initProject, playGame, playToken, resizeAllUserPool, resizeGlobalPool, setClusterConfig, update, withdraw } from './scripts';

program.version('0.0.1');

programCommand('initialize')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc } = cmd.opts();
        console.log('Solana Env Config:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);
        if (keypair === undefined || rpc === undefined) {
            console.log("Error Config Data Input");
            return;
        }
        await setClusterConfig(env, keypair, rpc);

        try {
            await initProject();
        } catch (e) {
            console.log(e);
        }
    });

programCommand('update')
    .option('-a, --address <string>', 'admin address')
    .option('-f, --fee <number>', 'set the loyalty fee[2.5 means 2.5%]')

    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc, address, fee } = cmd.opts();
        console.log('Solana Env Config:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);
        if (keypair === undefined || rpc === undefined) {
            console.log("Error Config Data Input");
            return;
        }

        await setClusterConfig(env, keypair, rpc);
        if (address === undefined) {
            console.log("Error Admin Address Input");
            return;
        }

        if (fee === undefined || isNaN(parseFloat(fee))) {
            console.log("Error Fee Input");
            return;
        }

        await update(address, parseFloat(fee));
    });

programCommand('withdraw')
    .option('-a, --amount <number>', 'amount to withdraw')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc, amount } = cmd.opts();
        console.log('Solana Env Config:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);
        if (keypair === undefined || rpc === undefined) {
            console.log("Error Config Data Input");
            return;
        }

        await setClusterConfig(env, keypair, rpc);
        if (amount === undefined || isNaN(parseInt(amount))) {
            console.log("Error Purpose Input");
            return;
        }

        await withdraw(parseFloat(amount));

    });

programCommand('play')
    .option('-s, --side <number>', 'head or tail, 0: tail, 1: head', '0')
    .option('-a, --amount <number>', 'amount to bet')
    .option('-t, --token <number>', 'token, 0: SOL, 1~9: tokens', '0')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc, amount, side, token } = cmd.opts();

        console.log('Solana Env Config:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);

        if (keypair === undefined || rpc === undefined) {
            console.log("Error Config Data Input");
            return;
        }

        await setClusterConfig(env, keypair, rpc);
        if (amount === undefined || isNaN(parseFloat(amount))) {
            console.log("Error Amount Input");
            return;
        }
        if (side != 0 && side != 1) {
            console.log("Error Side Input");
            return;
        }

        if (token == 0)
            await playGame(side, parseFloat(amount));
        else
            await playToken(side, parseFloat(amount), token - 1);
    });

programCommand('claim')
    .option('-t, --token <number>', 'token, 0: SOL, 1~9: tokens', '0')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc, token } = cmd.opts();

        console.log('Solana Cluster:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);
        await setClusterConfig(env, keypair, rpc);

        if (token == 0)
            await claim();
        else
            await claimToken(token - 1);
    });

programCommand('withdraw')
    .option('-a, --amount <number>', 'sol amount to withdraw')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc, amount } = cmd.opts();

        console.log('Solana Cluster:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);
        await setClusterConfig(env, keypair, rpc);

        if (amount == 0) {
            console.log("input withdraw amount");
            return;
        }

        await withdraw(amount);
    });

programCommand('status')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc } = cmd.opts();

        console.log('Solana Cluster:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);
        await setClusterConfig(env, keypair, rpc);

        console.log(await getGlobalInfo());
    });

programCommand('user-status')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .option('-a, --address <string>', 'user pubkey')
    .action(async (directory, cmd) => {
        const { env, keypair, rpc, address } = cmd.opts();

        console.log('Solana Cluster:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);
        await setClusterConfig(env, keypair, rpc);

        if (address === undefined) {
            console.log("Error User Address input");
            return;
        }
        console.log(await getUserInfo(new PublicKey(address)));
    });

programCommand('users-claimable')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc } = cmd.opts();

        console.log('Solana Cluster:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);
        await setClusterConfig(env, keypair, rpc);

        console.log(await getClaimableInfo());
    });

programCommand('resize')
    // eslint-disable-next-line @typescript-eslint/no-unused-vars
    .action(async (directory, cmd) => {
        const { env, keypair, rpc } = cmd.opts();

        console.log('Solana Cluster:', env);
        console.log('Keypair Path:', keypair);
        console.log('RPC URL:', rpc);
        await setClusterConfig(env, keypair, rpc);

        await resizeGlobalPool()
        // await resizeAllUserPool()
    });

function programCommand(name: string) {
    return program
        .command(name)
        .option('-e, --env <string>', 'Solana cluster env name', 'mainnet-beta')
        .option('-r, --rpc <string>', 'Solana cluster RPC name', 'https://api.mainnet-beta.solana.com')
        .option('-k, --keypair <string>', 'Solana wallet Keypair Path', '../key/G2.json')
}

program.parse(process.argv);

