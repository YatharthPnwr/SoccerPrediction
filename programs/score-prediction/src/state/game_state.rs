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
    pub seed: u64,
    pub admin_address: Pubkey,
    pub oracle_address: Pubkey,
    #[max_len(32)]
    pub team_a_name: String,
    #[max_len(32)]
    pub team_b_name: String,

    pub virtual_team_a_pool_tokens: u64,
    pub virtual_team_b_pool_tokens: u64,
    pub k: u128,

    pub vault_team_a: Pubkey,
    pub vault_team_b: Pubkey,
    pub vault_sol_a: u64,
    pub vault_sol_b: u64,

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
