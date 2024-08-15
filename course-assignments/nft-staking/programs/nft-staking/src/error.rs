use anchor_lang::prelude::*;

#[error_code]

pub enum ErrorCode {
    #[msg("Max stake reached")]
    MaxStake,
    #[msg("Cannot unstake yet")]
    CantUnstakeYet,
}