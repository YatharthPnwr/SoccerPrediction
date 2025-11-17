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
    #[msg("The Match has not yet ended")]
    MatchNotEndedYet,
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
    #[msg("Arithmetic overflow occurred")]
    Overflow,
    #[msg("Division by zero")]
    DivisionByZero,
    #[msg("No winning shares available")]
    NoWinningShares,
    #[msg("Unauthorized admin")]
    UnauthorizedAdmin,
    #[msg("Match is not live")]
    MatchNotLive,
    #[msg("Match has already started")]
    MatchAlreadyStarted,
}
