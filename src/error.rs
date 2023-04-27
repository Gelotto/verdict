use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Debug, Error)]
#[non_exhaustive]
pub enum ContractError {
  #[error("{0}")]
  Std(#[from] StdError),

  #[error("NotAuthorized")]
  NotAuthorized,

  #[error("ValidationError")]
  ValidationError,
}
