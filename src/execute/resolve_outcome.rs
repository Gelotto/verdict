use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response, Storage};

use crate::{
  error::ContractError,
  models::ContractResult,
  state::{
    load_candidate_count, load_config, process_outcome_resolution, require_active_game_status,
    require_sender_is_resolver,
  },
};

pub fn resolve_outcome(
  deps: DepsMut,
  env: Env,
  info: MessageInfo,
  candidate_indices: &Vec<u8>,
) -> ContractResult<Response> {
  require_active_game_status(deps.storage)?;
  require_sender_is_resolver(deps.storage, &info.sender)?;
  validate_request(deps.storage, &env, candidate_indices)?;
  process_outcome_resolution(deps.storage, candidate_indices)?;
  Ok(Response::new().add_attributes(vec![attr("action", "resolve_outcome")]))
}

fn validate_request(
  storage: &dyn Storage,
  env: &Env,
  candidate_indices: &Vec<u8>,
) -> ContractResult<()> {
  let config = load_config(storage)?;

  if env.block.time >= config.expires_at {
    // timed out
    return Err(ContractError::NotAuthorized);
  }

  if env.block.time < config.closes_at {
    // bests are still open
    return Err(ContractError::NotAuthorized);
  }

  if candidate_indices.len() > load_candidate_count(storage)? as usize {
    // candidate index out of bounds
    return Err(ContractError::NotAuthorized);
  }

  Ok(())
}
