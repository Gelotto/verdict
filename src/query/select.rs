use crate::{msg::SelectResponse, state::OWNER};
use cosmwasm_std::{Addr, Deps, StdResult};
use cw_repository::client::Repository;

pub fn select(
  deps: Deps,
  fields: Option<Vec<String>>,
  _wallet: Option<Addr>,
) -> StdResult<SelectResponse> {
  let loader = Repository::loader(deps.storage, &fields);
  Ok(SelectResponse {
    owner: loader.get("owner", &OWNER)?,
  })
}
