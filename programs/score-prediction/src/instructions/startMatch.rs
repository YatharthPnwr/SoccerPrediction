use crate::state::GameState;
use anchor_lang::prelude::*;
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct StartGame<'info> {
    #[account(mut)]
    pub admin_address: Signer<'info>,

    #[account(
        seeds= [b"gameState", seed.to_le_bytes().as_ref()], 
        bump
    )]
    pub game_state: Account<'info, GameState>,
}

impl<'info> StartGame<'info> {
    pub fn start_game(&mut self) -> Result<()> {
        //@note - complete this
        //Check if the signer is the admin
        //If the check is done,
        //set the game state to on.
        self.game_state.match_status = crate::MatchStatus::Live;
        Ok(())
    }
}
