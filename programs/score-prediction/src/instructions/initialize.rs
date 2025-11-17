use crate::state::GameState;
use anchor_lang::prelude::*;
use constant_product_curve::ConstantProduct;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub admin_address: Signer<'info>,

    /// CHECK: This is just an oracle address we're storing, no validation needed
    pub oracle_address: UncheckedAccount<'info>,

    #[account(
        init,
        payer = admin_address,
        space= GameState::DISCRIMINATOR.len() + GameState::INIT_SPACE,
        seeds= [b"gameState", seed.to_le_bytes().as_ref()], 
        bump
    )]
    pub game_state: Account<'info, GameState>,

    /// CHECK: PDA vault controlled by program
    #[account(
        seeds = [b"vault", seed.to_le_bytes().as_ref()],
        bump
    )]
    pub vault: AccountInfo<'info>,

    pub system_program: Program<'info, System>,
}

impl<'info> Initialize<'info> {
    pub fn initialize(
        &mut self,
        seed: u64,
        initial_virtual_pool_liqidity: u64,
        team_a_name: String,
        team_b_name: String,
        bumps: &InitializeBumps,
    ) -> Result<()> {
        //calculate k
        let k_initial = ConstantProduct::k_from_xy(
            initial_virtual_pool_liqidity,
            initial_virtual_pool_liqidity,
        )
        .unwrap();
        //@note- The checking of admin and oracle address is left
        //check if the admin is the address.
        //check for valid oracle address.
        self.game_state.set_inner(GameState {
            seed,
            vault_bump: bumps.vault,
            admin_address: self.admin_address.key(),
            oracle_address: self.oracle_address.key(),
            team_a_name,
            team_b_name,

            virtual_team_a_pool_tokens: initial_virtual_pool_liqidity,
            virtual_team_b_pool_tokens: initial_virtual_pool_liqidity,
            k: k_initial,

            vault: self.vault.key(),
            vault_sol_balance: 0,

            total_team_a_shares: 0,
            total_team_b_shares: 0,

            team_a_score: 0,
            team_b_score: 0,
            match_status: crate::MatchStatus::NotStarted,
        });
        Ok(())
    }
}
