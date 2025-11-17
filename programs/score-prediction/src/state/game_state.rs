use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, PartialEq, Eq, InitSpace)]
pub enum MatchStatus {
    Live,
    Ended,
    NotStarted,
}

#[account]
#[derive(InitSpace)]
pub struct GameState {
    pub seed: u64,              // Seed of the game
    pub vault_bump: u8,         // Bump seed for the vault PDA
    pub admin_address: Pubkey,  // admin address that will manage the game
    pub oracle_address: Pubkey, // The oracle that will update the scores
    #[max_len(32)]
    pub team_a_name: String, // Team A name
    #[max_len(32)]
    pub team_b_name: String, // Team B name

    pub virtual_team_a_pool_tokens: u64, // Virtual tokens of team A
    pub virtual_team_b_pool_tokens: u64, // Virtual tokens of team B
    pub k: u128,                         // vitual_a_tokens * virtual_b_tokens

    pub vault: Pubkey,          // Actual vault that holds sol
    pub vault_sol_balance: u64, // Total amount of sol in the vault

    pub total_team_a_shares: u64,
    pub total_team_b_shares: u64,
    pub team_a_score: u8,
    pub team_b_score: u8,
    pub match_status: MatchStatus,
}

#[account]
#[derive(InitSpace)]
pub struct MatchShares {
    pub team_a_shares: u64,
    pub team_b_shares: u64,
}
