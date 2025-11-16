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
}
