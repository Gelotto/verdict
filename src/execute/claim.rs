use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};
use cw_lib::utils::funds::build_send_submsg;

use crate::{
  models::ContractResult,
  state::{load_config, process_claim, require_non_active_game_status},
};

pub fn claim(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  require_non_active_game_status(deps.storage)?;

  let token = load_config(deps.storage)?.token;
  let claim_amount = process_claim(deps.storage, &info.sender)?;
  let mut resp = Response::new().add_attribute("action", "claim");

  if !claim_amount.is_zero() {
    resp = resp
      .add_attribute("amount", claim_amount.to_string())
      .add_submessage(build_send_submsg(&info.sender, claim_amount, &token)?)
  }

  Ok(resp)
}
