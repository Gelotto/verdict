use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Timestamp, Uint128};
use cw_lib::models::Token;

use crate::error::ContractError;

pub type ContractResult<T> = Result<T, ContractError>;

#[cw_serde]
pub enum GameStatus {
  Active,
  Canceled,
  Resolved,
}

#[cw_serde]
pub struct CandidateInitArgs {
  pub name: String,
  pub image_url: Option<String>,
  pub ticket_supply: Option<u32>,
}

#[cw_serde]
pub struct Candidate {
  pub name: String,
  pub ticket_count: u32,
  pub backer_count: u32,
  pub image_url: Option<String>,
  pub ticket_supply: Option<u32>,
}

#[cw_serde]
pub struct Config {
  pub token: Token,
  pub ticket_price: Uint128,
  pub closes_at: Timestamp,
  pub expires_at: Timestamp,
  pub resolver_addr: Addr,
}

#[cw_serde]
pub struct Account {
  pub has_claimed: bool,
  pub ticket_count: u32,
}

#[cw_serde]
pub struct Wager {
  pub candidate_index: u8,
  pub ticket_count: u32,
}

impl Wager {
  pub fn validate(
    &self,
    candidate_count: u8,
  ) -> ContractResult<()> {
    if self.ticket_count == 0 {
      return Err(ContractError::ValidationError {});
    }
    if self.candidate_index >= candidate_count {
      return Err(ContractError::ValidationError {});
    }
    if self.ticket_count == 0 {
      return Err(ContractError::ValidationError {});
    }
    Ok(())
  }
}
