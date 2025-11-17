use anchor_lang::prelude::*;
use anchor_lang::system_program::{transfer, Transfer};

use crate::errors::ErrorCode;
use crate::state::GameState;
use crate::{MatchShares, MatchStatus};
#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct ClaimRewards<'info> {
    //User that wants to claim the rewards
    #[account(mut)]
    pub user: Signer<'info>,

    //The users rewards PDA.
    #[account(
        mut,
        seeds=[b"matchShares",  seed.to_le_bytes().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_shares: Account<'info, MatchShares>,

    /// CHECK: PDA vault controlled by program
    #[account(
        mut,
        seeds = [b"vault", seed.to_le_bytes().as_ref()],
        bump
    )]
    pub vault: AccountInfo<'info>,

    //The game config
    #[account(
        mut,
        seeds= [b"gameState", seed.to_le_bytes().as_ref()], 
        bump
    )]
    pub game_state: Account<'info, GameState>,
    pub system_program: Program<'info, System>,
}

impl<'info> ClaimRewards<'info> {
    pub fn claim_rewards(&mut self, seed: u64) -> Result<()> {
        //Check if the game is over yet.
        require!(
            self.game_state.match_status == MatchStatus::Ended,
            ErrorCode::MatchNotEndedYet
        );
        //Find the winning team of the game from the config PDA
        let team_a_final_score = self.game_state.team_a_score;
        let team_b_final_score = self.game_state.team_b_score;
        let user_winning_team_shares = if team_a_final_score > team_b_final_score {
            self.user_shares.team_a_shares
        } else {
            self.user_shares.team_b_shares
        };

        //Find the share of sol the user deserves based on thier shares.
        let total_winning_shares = if team_a_final_score > team_b_final_score {
            self.game_state.total_team_a_shares
        } else {
            self.game_state.total_team_b_shares
        };
        //Send the deserving shares to the user from the vault.
        // Use the stored vault balance from game end, not current balance
        let user_reward = (user_winning_team_shares as u128)
            .checked_mul(self.game_state.vault_sol_balance as u128)
            .ok_or(ErrorCode::Overflow)?
            .checked_div(total_winning_shares as u128)
            .ok_or(ErrorCode::DivisionByZero)? as u64;
        //transfer the rewards to the user
        self.transfer_rewards(user_reward, self.user.to_account_info(), seed)?;

        self.user_shares.team_a_shares = 0;
        self.user_shares.team_b_shares = 0;
        Ok(())
    }

    pub fn transfer_rewards(
        &mut self,
        amount: u64,
        to: AccountInfo<'info>,
        seed: u64,
    ) -> Result<()> {
        let seed_bytes = seed.to_le_bytes();
        let bump = &[self.game_state.vault_bump];

        let signer_seeds: &[&[&[u8]]] = &[&[b"vault", seed_bytes.as_ref(), bump]];

        // Ensure vault maintains minimum rent-exempt balance
        let rent = Rent::get()?;
        let min_rent_exempt = rent.minimum_balance(0);
        let available_balance = self.vault.lamports().saturating_sub(min_rent_exempt);

        // Only transfer up to the available balance
        let transfer_amount = amount.min(available_balance);

        if transfer_amount > 0 {
            let accounts = Transfer {
                from: self.vault.to_account_info(),
                to: to.to_account_info(),
            };

            let cpi_ctx = CpiContext::new_with_signer(
                self.system_program.to_account_info(),
                accounts,
                signer_seeds,
            );

            transfer(cpi_ctx, transfer_amount)?;
        }

        Ok(())
    }
}
