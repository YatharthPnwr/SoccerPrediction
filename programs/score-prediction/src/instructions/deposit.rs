use crate::errors::ErrorCode;
use crate::state::GameState;
use crate::state::MatchShares;
use crate::MatchStatus;
use anchor_lang::prelude::*;
use anchor_lang::system_program::transfer;
use anchor_lang::system_program::Transfer;
use constant_product_curve::ConstantProduct;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Deposit<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        seeds= [b"gameState", seed.to_le_bytes().as_ref()], 
        bump
    )]
    pub game_state: Account<'info, GameState>,

    #[account(
        init_if_needed,
        payer=user,
        space= MatchShares::DISCRIMINATOR.len() + MatchShares::INIT_SPACE,
        seeds=[b"matchShares", seed.to_le_bytes().as_ref(), user.key().as_ref()],
        bump
    )]
    pub user_shares: Account<'info, MatchShares>,

    /// CHECK: This is the vault account for team A
    #[account(
        mut,
        seeds = [b"vault", seed.to_le_bytes().as_ref()],
        bump
    )]
    pub vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Deposit<'info> {
    pub fn deposit(&mut self, seed: u64, amount: u64, team_name: String) -> Result<()> {
        //Deposit logic here
        //Check if the match has started yet?
        require!(
            self.game_state.match_status == MatchStatus::Live,
            ErrorCode::MatchNotLiveYet
        );
        //Check if the team names are correct(Expensive)
        require!(
            team_name == self.game_state.team_a_name || team_name == self.game_state.team_b_name,
            ErrorCode::InvalidTeamName
        );

        //Check if the address of the vaults given are correct
        require!(
            self.vault.key() == self.game_state.vault,
            ErrorCode::InvalidVaultAddress
        );

        if team_name == self.game_state.team_a_name {
            let team_a_shares = self.game_state.virtual_team_a_pool_tokens;
            let team_b_shares = self.game_state.virtual_team_b_pool_tokens;

            let user_a_shares =
                ConstantProduct::delta_y_from_x_swap_amount(team_a_shares, team_b_shares, amount)
                    .unwrap();

            let updated_virtual_b_tokens: u64 =
                ConstantProduct::y2_from_x_swap_amount(team_a_shares, team_b_shares, amount)
                    .unwrap();

            let updated_virtual_a_tokens: u64 = team_a_shares + amount;

            self.deposit_sol_to_vault(amount, self.vault.to_account_info())?;

            //Update the game config PDA
            self.game_state.total_team_a_shares += user_a_shares;
            self.game_state.vault_sol_balance += amount;
            self.game_state.virtual_team_a_pool_tokens = updated_virtual_a_tokens;
            self.game_state.virtual_team_b_pool_tokens = updated_virtual_b_tokens;

            //update the users PDA
            self.user_shares.team_a_shares += user_a_shares;
        } else {
            let team_a_shares = self.game_state.virtual_team_a_pool_tokens;
            let team_b_shares = self.game_state.virtual_team_b_pool_tokens;

            let user_b_shares =
                ConstantProduct::delta_x_from_y_swap_amount(team_a_shares, team_b_shares, amount)
                    .unwrap();

            let updated_virtual_a_tokens: u64 =
                ConstantProduct::x2_from_y_swap_amount(team_a_shares, team_b_shares, amount)
                    .unwrap();

            let updated_virtual_b_tokens: u64 = team_b_shares + amount;

            self.deposit_sol_to_vault(amount, self.vault.to_account_info())?;

            //Update the game config PDA
            self.game_state.total_team_b_shares += user_b_shares;
            self.game_state.vault_sol_balance += amount;
            self.game_state.virtual_team_a_pool_tokens = updated_virtual_a_tokens;
            self.game_state.virtual_team_b_pool_tokens = updated_virtual_b_tokens;

            //update the users PDA
            self.user_shares.team_b_shares += user_b_shares;
        }

        Ok(())
    }

    pub fn deposit_sol_to_vault(&mut self, amount: u64, to: AccountInfo<'info>) -> Result<()> {
        let cpi_context = Transfer {
            from: self.user.to_account_info(),
            to: to,
        };

        let ctx = CpiContext::new(self.system_program.to_account_info(), cpi_context);

        transfer(ctx, amount)?;
        Ok(())
    }
}
