use crate::errors::ErrorCode;
use crate::state::GameState;
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct StartGame<'info> {
    #[account(mut)]
    pub admin_address: Signer<'info>,

    #[account(
        mut,
        seeds = [b"gameState", seed.to_le_bytes().as_ref()], 
        bump
    )]
    pub game_state: Account<'info, GameState>,
}

impl<'info> StartGame<'info> {
    pub fn start_game(&mut self, seed: u64) -> Result<()> {
        // Check if the signer is the admin
        require!(
            self.admin_address.key() == self.game_state.admin_address,
            ErrorCode::UnauthorizedAdmin
        );

        // Check if match is currently NotStarted
        require!(
            self.game_state.match_status == crate::MatchStatus::NotStarted,
            ErrorCode::MatchAlreadyStarted
        );

        // Set the game state to Live
        self.game_state.match_status = crate::MatchStatus::Live;
        Ok(())
    }
}
