use cosmwasm_std::{attr, DepsMut, Env, MessageInfo, Response};

use crate::{
  models::ContractResult,
  state::{process_cancellation, require_active_game_status, require_permission},
};

pub fn cancel(
  deps: DepsMut,
  _env: Env,
  info: MessageInfo,
) -> ContractResult<Response> {
  require_active_game_status(deps.storage)?;
  require_permission(&deps.as_ref(), &info.sender, "cancel")?;
  process_cancellation(deps.storage)?;
  Ok(Response::new().add_attributes(vec![attr("action", "cancel")]))
}
