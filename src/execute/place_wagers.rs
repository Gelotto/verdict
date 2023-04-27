use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Uint128};
use cw_lib::{
  models::Token,
  utils::funds::{build_cw20_transfer_from_msg, has_funds},
};

use crate::{
  error::ContractError,
  models::{Config, ContractResult, Wager},
  state::{load_candidate_count, load_config, process_wager, require_active_game_status},
};

pub fn place_wagers(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  address: Option<Addr>,
  wagers: &Vec<Wager>,
) -> ContractResult<Response> {
  let config = load_config(deps.storage)?;

  require_active_game_status(deps.storage)?;
  validate_request(&env, &config)?;

  let n_candidates = load_candidate_count(deps.storage)?;
  let buyer = address.unwrap_or(info.sender.clone());
  let mut payment_amount = Uint128::zero();

  // validate and place each wager
  for wager in wagers.iter() {
    wager.validate(n_candidates)?;
    process_wager(deps.storage, &buyer, wager)?;
    payment_amount += config.ticket_price * Uint128::from(wager.ticket_count);
  }

  let mut resp = Response::new().add_attributes(vec![attr("action", "place_wagers")]);

  // verify payment and add tranfer msg
  match &config.token {
    Token::Native { denom } => {
      if !has_funds(&info.funds, payment_amount, &denom) {
        return Err(ContractError::NotAuthorized);
      }
    },
    Token::Cw20 { address } => {
      resp = resp.add_message(build_cw20_transfer_from_msg(
        &info.sender,
        &env.contract.address,
        address,
        payment_amount,
      )?)
    },
  }

  Ok(resp)
}

fn validate_request(
  env: &Env,
  config: &Config,
) -> ContractResult<()> {
  if env.block.time >= config.closes_at {
    return Err(crate::error::ContractError::NotAuthorized);
  }

  Ok(())
}
