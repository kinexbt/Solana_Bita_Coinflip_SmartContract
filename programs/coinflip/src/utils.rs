use anchor_lang::prelude::*;
use anchor_lang::solana_program::{program::invoke, system_instruction::transfer};
// use sha2::Digest;
// use sha2::Sha256;
use solana_program::program::invoke_signed;
use std::cmp::Ordering;

// transfer sol
pub fn sol_transfer_with_signer<'a>(
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    signers: &[&[&[u8]]; 1],
    amount: u64,
) -> Result<()> {
    let ix = solana_program::system_instruction::transfer(source.key, destination.key, amount);
    invoke_signed(&ix, &[source, destination, system_program], signers)?;
    Ok(())
}

pub fn sol_transfer_user<'a>(
    source: AccountInfo<'a>,
    destination: AccountInfo<'a>,
    system_program: AccountInfo<'a>,
    amount: u64,
) -> Result<()> {
    let ix = solana_program::system_instruction::transfer(source.key, destination.key, amount);
    invoke(&ix, &[source, destination, system_program])?;
    Ok(())
}

pub fn puffed_out_string(s: &String, size: usize) -> String {
    let mut array_of_zeroes = vec![];

    let puff_amount = size - s.len();
    while array_of_zeroes.len() < puff_amount {
        array_of_zeroes.push(0u8);
    }
    s.clone() + std::str::from_utf8(&array_of_zeroes).unwrap()
}

//  Generate pseudo random number
pub struct HashStruct {
    pub nonce: u64,
    pub initial_seed: u64,
}

// unsafe fn any_as_u8_slice<T: Sized>(p: &T) -> &[u8] {
//     ::std::slice::from_raw_parts((p as *const T) as *const u8, ::std::mem::size_of::<T>())
// }

// pub fn get_rand(seed: u64, nonce: u64) -> u64 {
//     let hashstruct = HashStruct {
//         nonce,
//         initial_seed: seed,
//     };
//     let vec_to_hash = unsafe { self::any_as_u8_slice(&hashstruct) };
//     let hash = &(Sha256::new().chain_update(vec_to_hash).finalize()[..32]);

//     // hash is a vector of 32 8bit numbers.  We can take slices of this to generate our 4 random u64s
//     let mut hashed_randoms: [u64; 4] = [0; 4];
//     for i in 0..4 {
//         let hash_slice = &hash[i * 8..(i + 1) * 8];
//         hashed_randoms[i] =
//             u64::from_le_bytes(hash_slice.try_into().expect("slice with incorrect length"));
//     }

//     return hashed_randoms[2] % 1000000;
// }

pub fn resize_account<'info>(
    account_info: AccountInfo<'info>,
    new_space: usize,
    payer: AccountInfo<'info>,
    system_program: AccountInfo<'info>,
) -> Result<()> {
    let rent = Rent::get()?;
    let new_minimum_balance = rent.minimum_balance(new_space);
    let current_balance = account_info.lamports();

    match new_minimum_balance.cmp(&current_balance) {
        Ordering::Greater => {
            let lamports_diff = new_minimum_balance.saturating_sub(current_balance);
            invoke(
                &transfer(&payer.key(), &account_info.key(), lamports_diff),
                &[payer.clone(), account_info.clone(), system_program.clone()],
            )?;
        }
        Ordering::Less => {
            let lamports_diff = current_balance.saturating_sub(new_minimum_balance);
            **account_info.try_borrow_mut_lamports()? = new_minimum_balance;
            **payer.try_borrow_mut_lamports()? = payer
                .lamports()
                .checked_add(lamports_diff)
                .expect("Add error");
        }
        Ordering::Equal => {}
    }
    account_info.realloc(new_space, false)?;
    Ok(())
}
