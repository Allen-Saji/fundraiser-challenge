
use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    msg,
    program::invoke,
    program_pack::Pack,
    program_error::ProgramError,
    sysvar::{clock::Clock, Sysvar},
};
use spl_token::{
    instruction::transfer,
    state::Mint
};
// use std::convert::TryInto;

use crate::{
    state::{Fundraiser, Contributor},
    error::*,
    constants::*,
};

pub fn contribute(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    let accounts_iter = &mut accounts.iter();

    let contributor = next_account_info(accounts_iter)?;
    let mint_to_raise = next_account_info(accounts_iter)?;
    let fundraiser_account = next_account_info(accounts_iter)?;
    let contributor_account_info = next_account_info(accounts_iter)?;
    let contributor_ata = next_account_info(accounts_iter)?;
    let vault = next_account_info(accounts_iter)?;
    let token_program = next_account_info(accounts_iter)?;

    // Deserialize data for custom structs
    let mut fundraiser: Fundraiser = Fundraiser::try_from_slice(&fundraiser_account.data.borrow())?;
    let mut contributor_account: Contributor = Contributor::try_from_slice(&contributor_account_info.data.borrow())?;
    let amount = u64::from_le_bytes(instruction_data.try_into().unwrap());
    // Minimum contribution check
    let mint: Mint = Mint::unpack(&mint_to_raise.data.borrow())?;
    if amount <= 1_u64.pow(mint.decimals.into()) {
        msg!("Contribution too small");
        return Err(ProgramError::Custom(FundraiserError::ContributionTooSmall as u32));
    }

    // Maximum contribution check
    if amount > (fundraiser.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER {
        msg!("Contribution too big");
        return Err(ProgramError::Custom(FundraiserError::ContributionTooBig as u32));
    }

    // Fundraiser duration check
    let current_time = Clock::get()?.unix_timestamp;
    if fundraiser.duration <= ((current_time - fundraiser.time_started) / SECONDS_TO_DAYS) as u8 {
        msg!("Fundraiser has ended");
        return Err(ProgramError::Custom(FundraiserError::FundraiserEnded as u32));
    }

    // Maximum contribution per user check
    let max_contrib = (fundraiser.amount_to_raise * MAX_CONTRIBUTION_PERCENTAGE) / PERCENTAGE_SCALER;
    if contributor_account.amount > max_contrib || (contributor_account.amount + amount) > max_contrib {
        msg!("Maximum contributions reached");
        return Err(ProgramError::Custom(FundraiserError::MaximumContributionsReached as u32));
    }

    // Transfer funds from contributor to the vault
    let transfer_ix = transfer(
        token_program.key,
        contributor_ata.key,
        vault.key,
        contributor.key,
        &[],
        amount,
    )?;
    invoke(
        &transfer_ix,
        &[
            contributor_ata.clone(),
            vault.clone(),
            contributor.clone(),
            token_program.clone(),
        ],
    )?;

    // Update state data
    fundraiser.current_amount += amount;
    contributor_account.amount += amount;

    // Serialize state back to account data
    fundraiser.serialize(&mut *fundraiser_account.data.borrow_mut())?;
    fundraiser.serialize(&mut *contributor_account_info.data.borrow_mut())?;

    Ok(())
}