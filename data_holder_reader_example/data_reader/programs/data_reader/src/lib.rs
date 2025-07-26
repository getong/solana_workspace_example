use anchor_lang::prelude::*;

declare_id!("59cnY1rjumvuLbab9MRqFmk62QALDmjFRBzMEupqeyzH");

#[program]
pub mod data_reader {
    use super::*;

    pub fn read_other_data(ctx: Context<ReadOtherData>) -> Result<()> {
        let data_account = &ctx.accounts.other_data;

        if data_account.data_is_empty() {
            return err!(MyError::NoData);
        }

        let mut data_slice: &[u8] = &data_account.data.borrow();
        let data_struct: Storage = AccountDeserialize::try_deserialize(&mut data_slice)?;

        msg!("The value of x is: {}", data_struct.x);
        Ok(())
    }
}

#[error_code]
pub enum MyError {
    #[msg("No data")]
    NoData,
}

#[derive(Accounts)]
pub struct ReadOtherData<'info> {
    /// CHECK: We do not own this account
    other_data: UncheckedAccount<'info>,
}

#[account]
pub struct Storage {
    x: u64,
}
