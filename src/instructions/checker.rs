use borsh::BorshDeserialize;
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
    pubkey::Pubkey,
    
};
use spl_token::instruction as token_instruction;


use crate::state::Fundraiser;

pub fn check_contributions(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    // Get all account infos
    let maker_info = next_account_info(account_info_iter)?;
    // let mint_to_raise_info = next_account_info(account_info_iter)?;
    let fundraiser_info = next_account_info(account_info_iter)?;
    let vault_info = next_account_info(account_info_iter)?;
    let maker_ata_info = next_account_info(account_info_iter)?;
    let token_program_info = next_account_info(account_info_iter)?;
    // let system_program_info = next_account_info(account_info_iter)?;
    // let associated_token_program_info = next_account_info(account_info_iter)?;

    // Verify maker is signer
    if !maker_info.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Deserialize the fundraiser account
    let  fundraiser: Fundraiser = Fundraiser::try_from_slice(&fundraiser_info.data.borrow())?;

    // Verify the fundraiser PDA
    let (fundraiser_pda, bump_seed) = Pubkey::find_program_address(
        &[
            b"fundraiser",
            maker_info.key.as_ref(),
        ],
        program_id,
    );
    if fundraiser_pda != *fundraiser_info.key {
        return Err(ProgramError::InvalidSeeds);
    }

    // Verify vault is a valid associated token account
  

    // Verify maker ATA
   

    // Create maker ATA if it doesn't exist
   

    // Replace `vault_data.amount` with logic to get the amount
    let transfer_amount = fundraiser.amount_to_raise; // Replace with your actual field or logic if needed

    // Transfer tokens from vault to maker
    let transfer_ix = token_instruction::transfer(
        token_program_info.key,
        vault_info.key,
        maker_ata_info.key,
        &fundraiser_pda,
        &[],
        transfer_amount,
    )?;

    // Signs the transfer with the fundraiser PDA
    invoke_signed(
        &transfer_ix,
        &[
            vault_info.clone(),
            maker_ata_info.clone(),
            fundraiser_info.clone(),
            token_program_info.clone(),
        ],
        &[&[
            b"fundraiser",
            maker_info.key.as_ref(),
            &[bump_seed],
        ]],
    )?;

    // Close the fundraiser account by transferring its lamports to the maker
    let dest_starting_lamports = maker_info.lamports();
    **maker_info.lamports.borrow_mut() = dest_starting_lamports
        .checked_add(fundraiser_info.lamports())
        .ok_or(ProgramError::ArithmeticOverflow)?;
    **fundraiser_info.lamports.borrow_mut() = 0;

    // Clear the fundraiser data
    fundraiser_info.data.borrow_mut().fill(0);

    Ok(())
}
