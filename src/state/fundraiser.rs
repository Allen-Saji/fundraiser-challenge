use borsh::{BorshDeserialize, BorshSerialize};
use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

#[derive(BorshSerialize, BorshDeserialize)]
pub struct Fundraiser {
    pub maker: Pubkey,
    pub mint_to_raise: Pubkey,
    pub amount_to_raise: u64,
    pub current_amount: u64,
    pub time_started: i64,
    pub duration: u8,
    pub bump: u8,
}

impl Fundraiser {
    pub const LEN: usize = 32 + // maker
                          32 + // mint_to_raise
                          8 +  // amount_to_raise
                          8 +  // current_amount
                          8 +  // time_started
                          1 +  // duration
                          1;   // bump

    pub fn init(
        account: &AccountInfo,
        maker: Pubkey,
        mint_to_raise: Pubkey,
        amount_to_raise: u64,
        duration: u8,
        bump: u8,
        time_started: i64,
    ) -> Result<(), ProgramError> {
        let fundraiser = Fundraiser {
            maker,
            mint_to_raise,
            amount_to_raise,
            current_amount: 0,
            time_started,
            duration,
            bump,
        };

        fundraiser.serialize(&mut *account.try_borrow_mut_data()?)?;
        Ok(())
    }
}