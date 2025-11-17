use crate::errors::ErrorCode;
use crate::state::GameState;
use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct EndGame<'info> {
    #[account(mut)]
    pub admin_address: Signer<'info>,

    #[account(
        mut,
        seeds = [b"gameState", seed.to_le_bytes().as_ref()], 
        bump
    )]
    pub game_state: Account<'info, GameState>,

    /// CHECK: PDA vault controlled by program
    #[account(
        mut,
        seeds = [b"vault", seed.to_le_bytes().as_ref()],
        bump
    )]
    pub vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> EndGame<'info> {
    pub fn end_game(&mut self, seed: u64) -> Result<()> {
        // Check if the signer is the admin
        require!(
            self.admin_address.key() == self.game_state.admin_address,
            ErrorCode::UnauthorizedAdmin
        );

        // Check if match is currently Live
        require!(
            self.game_state.match_status == crate::MatchStatus::Live,
            ErrorCode::MatchNotLive
        );

        // Calculate 5% platform fee
        let vault_balance = self.vault.lamports();
        let platform_fee = vault_balance
            .checked_mul(5)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(100)
            .ok_or(ErrorCode::DivisionByZero)?;

        // Transfer 5% platform fee to admin
        if platform_fee > 0 {
            self.transfer_platform_fee(platform_fee, seed)?;
        }

        // Update vault balance in game state
        self.game_state.vault_sol_balance = self
            .game_state
            .vault_sol_balance
            .checked_sub(platform_fee)
            .ok_or(ErrorCode::Overflow)?;

        // Set the game state to ended
        self.game_state.match_status = crate::MatchStatus::Ended;

        Ok(())
    }

    pub fn transfer_platform_fee(&mut self, amount: u64, seed: u64) -> Result<()> {
        let seed_bytes = seed.to_le_bytes();
        let bump = &[self.game_state.vault_bump];

        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", seed_bytes.as_ref(), bump]];

        let accounts = Transfer {
            from: self.vault.to_account_info(),
            to: self.admin_address.to_account_info(),
        };

        let cpi_ctx = CpiContext::new_with_signer(
            self.system_program.to_account_info(),
            accounts,
            signer_seeds,
        );

        transfer(cpi_ctx, amount)?;
        Ok(())
    }
}
