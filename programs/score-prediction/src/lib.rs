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
        seed: u64,
        initial_virtual_pool_liqidity: u64,
        team_a_name: String,
        team_b_name: String,
    ) -> Result<()> {
        ctx.accounts.initialize(
            seed,
            initial_virtual_pool_liqidity,
            team_a_name,
            team_b_name,
            &ctx.bumps,
        )?;
        Ok(())
    }

    pub fn start_game(ctx: Context<StartGame>, seed: u64) -> Result<()> {
        ctx.accounts.start_game(seed)?;
        Ok(())
    }

    pub fn end_game(ctx: Context<EndGame>, seed: u64) -> Result<()> {
        ctx.accounts.end_game(seed)?;
        Ok(())
    }
    pub fn update_score(
        ctx: Context<UpdateScore>,
        seed: u64,
        new_team_a_score: u8,
        new_team_b_score: u8,
    ) -> Result<()> {
        ctx.accounts
            .update_score(seed, new_team_a_score, new_team_b_score)?;
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, seed: u64, amount: u64, team_name: String) -> Result<()> {
        ctx.accounts.deposit(seed, amount, team_name)?;
        Ok(())
    }

    pub fn claim_rewards(ctx: Context<ClaimRewards>, seed: u64) -> Result<()> {
        ctx.accounts.claim_rewards(seed)?;
        Ok(())
    }
}
