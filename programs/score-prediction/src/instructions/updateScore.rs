use std::ops::Mul;

use crate::errors::ErrorCode;
use crate::state::{GameState, MatchStatus};
use anchor_lang::prelude::*;

#[derive(Accounts)]
#[instruction(seed: u64)]
pub struct UpdateScore<'info> {
    /// Oracle/Admin who can update scores
    #[account(mut)]
    pub oracle: Signer<'info>,

    /// Match state to update
    #[account(
        mut,  // ✓ Must be mutable to update scores
        seeds = [b"gameState", seed.to_le_bytes().as_ref()], 
        bump,
        constraint = oracle.key() == game_state.oracle_address @ ErrorCode::UnauthorizedOracle,
    )]
    pub game_state: Account<'info, GameState>,
}

impl<'info> UpdateScore<'info> {
    pub fn update_score(&mut self, new_team_a_score: u8, new_team_b_score: u8) -> Result<()> {
        let game_state = &mut self.game_state;
        // 1. Check match is Live
        require!(
            game_state.match_status == MatchStatus::Live,
            ErrorCode::MatchNotLiveYet
        );

        // 3. Scores can only increase (anti-cheat)
        require!(
            new_team_a_score >= game_state.team_a_score,
            ErrorCode::ScoreCannotDecrease
        );

        require!(
            new_team_b_score >= game_state.team_b_score,
            ErrorCode::ScoreCannotDecrease
        );

        // 4. At least one score must change
        require!(
            new_team_a_score != game_state.team_a_score
                || new_team_b_score != game_state.team_b_score,
            ErrorCode::NoScoreChange
        );

        // 5. Reasonable score limits (prevent overflow)
        require!(
            new_team_a_score <= 50 && new_team_b_score <= 50,
            ErrorCode::ScoreTooHigh
        );

        let old_team_a_score = game_state.team_a_score;
        let old_team_b_score = game_state.team_b_score;

        // Calculate goal difference (can be negative)
        let goal_diff: i16 = (new_team_a_score as i16) - (new_team_b_score as i16);

        let (mult_a, mult_b) = Self::get_multipliers(goal_diff);

        let old_virtual_a = game_state.virtual_team_a_pool_tokens;
        let old_virtual_b = game_state.virtual_team_b_pool_tokens;
        let old_k = (old_virtual_a as u128)
            .checked_mul(old_virtual_b as u128)
            .ok_or(ErrorCode::MathOverflow)?;

        //APPLY MULTIPLIERS TO VIRTUAL POOLS

        // Formula: new_virtual = (old_virtual × multiplier) ÷ 10000
        let new_virtual_a = ((old_virtual_a as u128)
            .checked_mul(mult_a as u128)
            .ok_or(ErrorCode::MathOverflow)?)
        .checked_div(10_000)
        .ok_or(ErrorCode::MathOverflow)? as u64;

        let new_virtual_b = ((old_virtual_b as u128)
            .checked_mul(mult_b as u128)
            .ok_or(ErrorCode::MathOverflow)?)
        .checked_div(10_000)
        .ok_or(ErrorCode::MathOverflow)? as u64;

        //    RECALCULATE K (CONSTANT PRODUCT CHANGES!)

        let new_k = (new_virtual_a as u128)
            .checked_mul(new_virtual_b as u128)
            .ok_or(ErrorCode::MathOverflow)?;

        //    UPDATE MATCH STATE

        game_state.team_a_score = new_team_a_score;
        game_state.team_b_score = new_team_b_score;
        game_state.virtual_team_a_pool_tokens = new_virtual_a;
        game_state.virtual_team_b_pool_tokens = new_virtual_b;
        game_state.k = new_virtual_a.mul(new_virtual_b) as u128;

        emit!(ScoreUpdated {
            match_id: game_state.seed,
            oracle: self.oracle.key(),
            old_score_a: old_team_a_score,
            new_score_a: new_team_a_score,
            old_score_b: old_team_b_score,
            new_score_b: new_team_b_score,
            goal_diff,
            old_virtual_a,
            new_virtual_a,
            old_virtual_b,
            new_virtual_b,
            old_k,
            new_k,
            mult_a,
            mult_b,
            timestamp: Clock::get()?.unix_timestamp,
        });

        Ok(())
    }

    /// Get multipliers based on goal difference
    /// Returns: (team_a_multiplier, team_b_multiplier) in basis points
    ///
    /// # Multiplier Table (Hardcore)
    /// ```
    /// ╔═══════════╦════════════╦═══════════╗
    /// ║ Goal Diff ║ Team A Mult║ Team B Mult║
    /// ╠═══════════╬════════════╬═══════════╣
    /// ║   ≥ +3    ║   14000    ║   6000    ║
    /// ║    +2     ║   13500    ║   6500    ║
    /// ║    +1     ║   12500    ║   7500    ║
    /// ║     0     ║   10000    ║  10000    ║
    /// ║    -1     ║    7500    ║  12500    ║
    /// ║    -2     ║    6500    ║  13500    ║
    /// ║   ≤ -3    ║    6000    ║  14000    ║
    /// ╚═══════════╩════════════╩═══════════╝
    /// ```
    fn get_multipliers(goal_diff: i16) -> (u16, u16) {
        match goal_diff {
            // Team A winning scenarios
            d if d >= 3 => (14_000, 6_000), // Team A dominates: +140%, +60%
            2 => (13_500, 6_500),           // Team A ahead by 2: +135%, +65%
            1 => (12_500, 7_500),           // Team A ahead by 1: +125%, +75%

            // Draw scenario
            0 => (10_000, 10_000), // Equal: 100%, 100% (no change)

            // Team B winning scenarios
            -1 => (7_500, 12_500),           // Team B ahead by 1: +75%, +125%
            -2 => (6_500, 13_500),           // Team B ahead by 2: +65%, +135%
            d if d <= -3 => (6_000, 14_000), // Team B dominates: +60%, +140%

            // Fallback (shouldn't happen with validation)
            _ => (10_000, 10_000),
        }
    }
}
#[event]
pub struct ScoreUpdated {
    pub match_id: u64,
    pub oracle: Pubkey,

    // Score changes
    pub old_score_a: u8,
    pub new_score_a: u8,
    pub old_score_b: u8,
    pub new_score_b: u8,
    pub goal_diff: i16,

    // Virtual pool changes
    pub old_virtual_a: u64,
    pub new_virtual_a: u64,
    pub old_virtual_b: u64,
    pub new_virtual_b: u64,

    // Constant product changes
    pub old_k: u128,
    pub new_k: u128,

    // Multipliers applied
    pub mult_a: u16,
    pub mult_b: u16,

    pub timestamp: i64,
}
