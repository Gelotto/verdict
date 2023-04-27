use cosmwasm_std::{attr, Addr, DepsMut, Env, MessageInfo, Response, Storage};

use crate::{
  models::{ContractResult, Wager},
  state::{load_candidate_count, load_config, process_wager, require_active_game_status},
};

pub fn place_wagers(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  address: Option<Addr>,
  wagers: &Vec<Wager>,
) -> ContractResult<Response> {
  require_active_game_status(deps.storage)?;
  validate_request(deps.storage, &env)?;

  let n_candidates = load_candidate_count(deps.storage)?;
  let buyer = address.unwrap_or(info.sender);

  // validate and place each wager
  for wager in wagers.iter() {
    wager.validate(n_candidates)?;
    process_wager(deps.storage, &buyer, wager)?;
  }

  Ok(Response::new().add_attributes(vec![attr("action", "place_wagers")]))
}

fn validate_request(
  storage: &dyn Storage,
  env: &Env,
) -> ContractResult<()> {
  let config = load_config(storage)?;

  if env.block.time >= config.closes_at {
    return Err(crate::error::ContractError::NotAuthorized);
  }

  Ok(())
}
