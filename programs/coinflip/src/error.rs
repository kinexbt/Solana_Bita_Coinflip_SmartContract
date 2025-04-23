use anchor_lang::prelude::*;

#[error_code]
pub enum GameError {
    #[msg("Invalid bet amount")] // 6000
    InvalidBetAmount,
    #[msg("Invalid bet amount violating MaxWinAmount")] // 6001
    InvalidBetAmountMaxWinAmountViolation,
    #[msg("Insufficient User SOL Balance")] // 6002
    InsufficientUserBalance,
    #[msg("Insufficient Casino Bank SOL Balance")] // 6003
    InsufficientCasinoVault,
    #[msg("Mismatching Round Number")] // 6004
    RoundNumMismatch,
    #[msg("Not allowed to double bet")] // 6005
    NotAllowedDoubleBet,
    #[msg("Not Original Player")] // 6006
    NotOriginalPlayer,
    #[msg("Not Allowed Game Status")] // 6007
    NotAllowedStatus,
    #[msg("Invalid RTP")] // 6008
    InvalidRtp,
    #[msg("Only Operation Admin can call this")] // 6009
    UnauthorizedOperator,
    #[msg("Only Financial Admin can call this")] // 6010
    UnauthorizedFinanceAdmin,
    #[msg("Only Update Admin can call this")] // 6011
    UnauthorizedUpdateAdmin,
}
