use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Uint128};
use cw_lib::models::{Owner, Token};

use crate::models::{CandidateInitArgs, Config, Wager};

#[cw_serde]
pub struct InstantiateMsg {
  pub owner: Option<Owner>,
  pub candidates: Vec<CandidateInitArgs>,
  pub config: Config,
  pub acl_addr: Addr,
  pub ticket_price: Uint128,
  pub token: Token,
}

#[cw_serde]
pub enum ExecuteMsg {
  PlaceWagers {
    address: Option<Addr>,
    wagers: Vec<Wager>,
  },
  ResolveOutcome {
    candidates: Vec<u8>,
  },
  Cancel {},
}

#[cw_serde]
pub enum QueryMsg {
  Select {
    fields: Option<Vec<String>>,
    wallet: Option<Addr>,
  },
}

#[cw_serde]
pub struct MigrateMsg {}

#[cw_serde]
pub struct SelectResponse {
  pub owner: Option<Owner>,
}
