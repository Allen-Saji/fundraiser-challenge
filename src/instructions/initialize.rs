use borsh::BorshDeserialize;
use solana_program::{
    account_info::AccountInfo,
    program::{invoke, invoke_signed},
    program_error::ProgramError,
    program_pack::Pack,
    pubkey::Pubkey,
    rent::Rent,
    system_instruction::create_account,
    system_program,
    sysvar::Sysvar,
    clock::Clock,
};
use spl_token::state::Mint;
use spl_associated_token_account::instruction::create_associated_token_account;

use crate::{
    state::Fundraiser,
    constants::{
        AMOUNT_OFFSET,
        DURATION_OFFSET,
        MIN_AMOUNT_TO_RAISE,
    },
    ID,
};

pub fn process_initialize(
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> Result<(), ProgramError> {
    let [
        maker,
        fundraiser,
        mint_to_raise,
        vault,
        system_program,
        token_program,
        associated_token_program,
    ] = accounts else {
        return Err(ProgramError::NotEnoughAccountKeys);
    };

    // Deserialize instruction data
    let amount = u64::try_from_slice(&instruction_data[..AMOUNT_OFFSET])?;
    let duration = u8::try_from_slice(&instruction_data[AMOUNT_OFFSET..DURATION_OFFSET])?;

    // Verify accounts
    if !maker.is_signer {
        return Err(ProgramError::MissingRequiredSignature);
    }

    // Derive and verify PDA
    let (_fundraiser_pda, bump) = Pubkey::find_program_address(
        &[b"fundraiser", maker.key.as_ref()],
        &ID,
    );


    if fundraiser.owner != &system_program::ID {
        return Err(ProgramError::AccountAlreadyInitialized);
    }

    // Verify minimum amount
    let mint_data = Mint::unpack(&mint_to_raise.try_borrow_data()?)?;
    if amount <= MIN_AMOUNT_TO_RAISE.pow(mint_data.decimals as u32) {
        return Err(ProgramError::InvalidArgument);
    }

    // Create fundraiser account
    let minimum_balance = Rent::get()?.minimum_balance(Fundraiser::LEN);
    let init_ix = create_account(
        maker.key,
        fundraiser.key,
        minimum_balance,
        Fundraiser::LEN as u64,
        &ID,
    );

    invoke_signed(
        &init_ix,
        &[maker.clone(), fundraiser.clone()],
        &[&[
            b"fundraiser",
            maker.key.as_ref(),
            &[bump],
        ]],
    )?;

    // Initialize fundraiser data
    Fundraiser::init(
        fundraiser,
        *maker.key,
        *mint_to_raise.key,
        amount,
        duration,
        bump,
        Clock::get()?.unix_timestamp,
    )?;


    // Create associated token account for vault
    invoke(
        &create_associated_token_account(
            maker.key,
            fundraiser.key,
            mint_to_raise.key,
            token_program.key,
        ),
        &[
            maker.clone(),
            vault.clone(),
            fundraiser.clone(),
            mint_to_raise.clone(),
            system_program.clone(),
            token_program.clone(),
            associated_token_program.clone(),
        ],
    )?;

    Ok(())
}