use anchor_lang::prelude::*;

pub mod errors;
pub mod instructions;
pub mod state;
pub use instructions::*;
pub use state::*;
declare_id!("5vNvacWDvy6asU5gM8Av1VZHT8NGM2gJE6QAqQ9DzFKC");

#[program]
pub mod score_prediction {
    use super::*;
    //Admin operations
    pub fn initialize(
        ctx: Context<Initialize>,
        initial_virtual_pool_liqidity: u64,
        team_a_name: String,
        team_b_name: String,
        seed: u64,
    ) -> Result<()> {
        ctx.accounts.initialize(
            initial_virtual_pool_liqidity,
            team_a_name,
            team_b_name,
            seed,
        )?;
        Ok(())
    }

    pub fn start_game(ctx: Context<StartGame>) -> Result<()> {
        ctx.accounts.start_game()?;
        Ok(())
    }

    pub fn end_game(ctx: Context<EndGame>) -> Result<()> {
        ctx.accounts.end_game()?;
        Ok(())
    }
    pub fn update_score(ctx: Context<UpdateScore>) -> Result<()> {
        ctx.accounts.update_score()?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64, team_name: String) -> Result<()> {
        ctx.accounts.deposit(amount, team_name)?;
        Ok(())
    }
}
