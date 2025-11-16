use anchor_lang::error_code;
use constant_product_curve::CurveError;

#[error_code]
pub enum ErrorCode {
    #[msg("Invalid Team Name given")]
    InvalidTeamName,
    #[msg("Invalid Account Info of vault sent")]
    InvalidVaultAddress,
    #[msg("The Match is not live yet")]
    MatchNotLiveYet,
    #[msg("This is an anauthorized oracle")]
    UnauthorizedOracle,
    #[msg("The new score cannot be less than the previous one")]
    ScoreCannotDecrease,
    #[msg("There was no change in the score")]
    NoScoreChange,
    #[msg("Score is too high")]
    ScoreTooHigh,
    #[msg("Mathematical Overflow")]
    MathOverflow,
}
