use borsh::{BorshSerialize, BorshDeserialize};
use solana_program::{
    account_info::{next_account_info, AccountInfo},
    entrypoint::ProgramResult,
    program::invoke_signed,
    program_error::ProgramError,
};
use spl_token::instruction::transfer;
use crate::state::{Contributor, Fundraiser};
   
pub fn refund_instruction(
    accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    let account_info_iter = &mut accounts.iter();

    let contributor = next_account_info(account_info_iter)?;
    let maker = next_account_info(account_info_iter)?;
    let fundraiser_account = next_account_info(account_info_iter)?;
    let contributor_account_info = next_account_info(account_info_iter)?;
    let contributor_ata = next_account_info(account_info_iter)?;
    let vault = next_account_info(account_info_iter)?;
    let token_program = next_account_info(account_info_iter)?;
    

    // Ensure the contributor has signed the transaction
    if !contributor.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Deserialize the Fundraiser and Contributor state
    let mut fundraiser = Fundraiser::try_from_slice(&fundraiser_account.data.borrow())?;
    let  contributor_account = Contributor::try_from_slice(&contributor_account_info.data.borrow())?;


    // Transfer funds from the vault to the contributor's ATA
    let transfer_ix = transfer(
        token_program.key,
        vault.key,
        contributor_ata.key,
        fundraiser_account.key,
        &[],
        contributor_account.amount,
    )?;

    let signer_seeds: &[&[&[u8]]] = &[&[
        b"fundraiser",
        maker.key.as_ref(),
        &[fundraiser.bump],
    ]];

    invoke_signed(
        &transfer_ix,
        &[
            vault.clone(),
            contributor_ata.clone(),
            fundraiser_account.clone(),
            token_program.clone(),
        ],
        signer_seeds,
    )?;

    // Update state: reduce the current amount in the fundraiser
    fundraiser.current_amount -= contributor_account.amount;

    // Save the updated state back to the account data
    fundraiser.serialize(&mut &mut fundraiser_account.data.borrow_mut()[..])?;
    contributor_account.serialize(&mut &mut contributor_account_info.data.borrow_mut()[..])?;

    Ok(())
}
