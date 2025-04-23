# Bita-Coinflip-Contract

## Install Dependencies
- Install `node` and `yarn`
- Install `ts-node` as global command
- Confirm the solana wallet preparation: `/home/---/.config/solana/id.json` in test case

## Usage
- Main script source for all functionality is here: `/cli/script.ts`
- Program account types are declared here: `/cli/types.ts`
- Idl to make the JS binding easy is here: `/cli/coinflip.ts`

Able to test the script functions working in this way.
- Change commands properly in the main functions of the `script.ts` file to call the other functions
- Confirm the `ANCHOR_WALLET` environment variable of the `ts-node` script in `package.json`
- Run `yarn ts-node`

# Features

##  How to deploy this program?
First of all, you have to git clone in your PC.
In the folder `coinflip`, in the terminal 
1. `yarn`

2. `anchor build`
3. `solana-keygen pubkey ../keys/G2.json`
4. You can get the pubkey of the `program ID : ex."5N...x6k"`
5. Please add this pubkey to the lib.rs
  `line 16: declare_id!("5N...x6k");`
6. Please add this pubkey to the Anchor.toml
  `line 4: coinflip = "5N...x6k"`
7. Please add this pubkey to the types.ts
  `line 6: export const STAKING_PROGRAM_ID = new PublicKey("5N...x6k");`
  
8. `anchor build` again
9. `anchor deploy`

<p align = "center">
Then, you can enjoy this program 🎭
</p>
</br>