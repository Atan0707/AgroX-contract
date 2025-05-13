use anchor_lang::prelude::*;

declare_id!("Go8PqyzmSG4LfAoo1PqGRbgctXynnRYAVa9RQx9aZRHC");

#[program]
pub mod agrox_contract {
    use super::*;

    pub fn initialize(ctx: Context<Initialize>) -> Result<()> {
        msg!("Greetings from: {:?}", ctx.program_id);
        Ok(())
    }
}

#[derive(Accounts)]
pub struct Initialize {}
