use std::collections::HashSet;

use crate::models::{Account, Config, ContractResult, GameStatus, Wager};
use crate::msg::InstantiateMsg;
use crate::{error::ContractError, models::Candidate};
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Storage, Uint128};
use cw_acl::client::Acl;
use cw_lib::models::Owner;
use cw_storage_plus::{Deque, Item, Map};

pub const OWNER: Item<Owner> = Item::new("owner");
pub const CONFIG: Item<Config> = Item::new("config");
pub const STATUS: Item<GameStatus> = Item::new("status");
pub const TOTAL_TICKET_COUNT: Item<u32> = Item::new("total_ticket_count");
pub const TICKET_PRICE: Item<Uint128> = Item::new("ticket_price");
pub const ACCOUNTS: Map<Addr, Account> = Map::new("accounts");
pub const CANDIDATE_COUNT: Item<u8> = Item::new("candidate_count");
pub const CANDIDATES: Map<u8, Candidate> = Map::new("candidates");
pub const OUTCOME_INDICES: Deque<u8> = Deque::new("outcome_indices");
pub const ACCOUNT_TICKET_COUNTS: Map<(Addr, u8), u32> = Map::new("account_ticket_counts");

/// Initialize contract state data.
pub fn initialize(
  deps: DepsMut,
  _env: &Env,
  info: &MessageInfo,
  msg: &InstantiateMsg,
) -> Result<(), ContractError> {
  validate_instiantiate_msg(&deps, msg)?;
  STATUS.save(deps.storage, &GameStatus::Active)?;
  CONFIG.save(deps.storage, &msg.config)?;
  TOTAL_TICKET_COUNT.save(deps.storage, &0)?;
  TICKET_PRICE.save(deps.storage, &msg.ticket_price)?;
  OWNER.save(
    deps.storage,
    &msg
      .owner
      .clone()
      .unwrap_or_else(|| Owner::Acl(info.sender.clone())),
  )?;
  Ok(())
}

pub fn validate_instiantiate_msg(
  deps: &DepsMut,
  msg: &InstantiateMsg,
) -> ContractResult<()> {
  deps.api.addr_validate(msg.config.resolver_addr.as_str())?;

  if let Some(owner) = &msg.owner {
    deps.api.addr_validate(
      match owner {
        Owner::Address(addr) => addr,
        Owner::Acl(addr) => addr,
      }
      .as_str(),
    )?;
  }

  if msg.ticket_price.is_zero() {
    return Err(ContractError::NotAuthorized {});
  }

  if msg.candidates.len() < 2 {
    return Err(ContractError::NotAuthorized {});
  }

  let mut visited_candidate_names: HashSet<String> = HashSet::with_capacity(msg.candidates.len());

  for c in msg.candidates.iter() {
    if visited_candidate_names.contains(&c.name.to_lowercase()) {
      return Err(ContractError::NotAuthorized {});
    } else {
      visited_candidate_names.insert(c.name.to_lowercase());
    }
    if c.name.is_empty() || c.name.len() > 100 {
      return Err(ContractError::NotAuthorized {});
    }
    if let Some(url) = &c.image_url {
      if url.is_empty() || url.len() > 256 {
        return Err(ContractError::NotAuthorized);
      }
    }
    if let Some(n) = c.ticket_supply {
      if n == 0 {
        return Err(ContractError::NotAuthorized);
      }
    }
  }

  Ok(())
}

pub fn require_permission(
  deps: &Deps,
  principal: &Addr,
  action: &str,
) -> ContractResult<()> {
  let is_allowed = match OWNER.load(deps.storage)? {
    Owner::Address(addr) => *principal == addr,
    Owner::Acl(acl_addr) => {
      let acl = Acl::new(&acl_addr);
      acl.is_allowed(&deps.querier, principal, action)?
    },
  };
  if !is_allowed {
    return Err(ContractError::NotAuthorized);
  }
  Ok(())
}

pub fn require_sender_is_resolver(
  storage: &dyn Storage,
  addr: &Addr,
) -> ContractResult<()> {
  if CONFIG.load(storage)?.resolver_addr != *addr {
    return Err(ContractError::NotAuthorized);
  }
  Ok(())
}

pub fn load_config(storage: &dyn Storage) -> ContractResult<Config> {
  Ok(CONFIG.load(storage)?)
}

pub fn load_candidate_count(storage: &dyn Storage) -> ContractResult<u8> {
  Ok(CANDIDATE_COUNT.load(storage)?)
}

pub fn load_game_status(storage: &dyn Storage) -> ContractResult<GameStatus> {
  Ok(STATUS.load(storage)?)
}

pub fn require_active_game_status(storage: &dyn Storage) -> ContractResult<()> {
  if load_game_status(storage)? != GameStatus::Active {
    return Err(ContractError::ValidationError);
  }
  Ok(())
}

pub fn require_non_active_game_status(storage: &dyn Storage) -> ContractResult<()> {
  if load_game_status(storage)? == GameStatus::Active {
    return Err(ContractError::ValidationError);
  }
  Ok(())
}

pub fn increment_wager_total(
  storage: &mut dyn Storage,
  buyer: &Addr,
  wager: &Wager,
) -> ContractResult<(u32, bool)> {
  let mut is_first_wager_for_candidate = false;

  let new_total = ACCOUNT_TICKET_COUNTS.update(
    storage,
    (buyer.clone(), wager.candidate_index),
    |maybe_n| -> ContractResult<_> {
      if let Some(n) = maybe_n {
        Ok(n + wager.ticket_count)
      } else {
        is_first_wager_for_candidate = true;
        Ok(wager.ticket_count)
      }
    },
  )?;

  Ok((new_total, is_first_wager_for_candidate))
}

pub fn process_wager(
  storage: &mut dyn Storage,
  buyer: &Addr,
  wager: &Wager,
) -> ContractResult<(Candidate, u32)> {
  // init an account for the buyer
  ACCOUNTS.update(
    storage,
    buyer.clone(),
    |maybe_account| -> ContractResult<_> {
      let mut account = maybe_account.unwrap_or_else(|| Account {
        has_claimed: false,
        ticket_count: 0,
      });
      account.ticket_count += wager.ticket_count;
      Ok(account)
    },
  )?;

  // increment the account's total tally for candidate specified by the wager,
  // and return a flag indicating whether this is the buyer's first wager,
  // specifically for this candidate.
  let (total_wager_size, is_first_wager_for_candidate) =
    increment_wager_total(storage, buyer, wager)?;

  // update candidate data (number of backers, wager supply, etc.)
  let candidate = CANDIDATES.update(
    storage,
    wager.candidate_index,
    |maybe_candidate| -> ContractResult<_> {
      if let Some(mut candidate) = maybe_candidate {
        // make sure tickets for the candidate aren't sold out
        if let Some(supply) = candidate.ticket_supply {
          if wager.ticket_count <= supply {
            candidate.ticket_supply = Some(supply - wager.ticket_count);
          } else {
            return Err(ContractError::ValidationError);
          }
        }
        // increment counter for distinct wallets with wagers on this candidate.
        if is_first_wager_for_candidate {
          candidate.backer_count += 1;
        }
        candidate.ticket_count += wager.ticket_count;
        Ok(candidate)
      } else {
        // should never get here. candidate not found
        return Err(ContractError::ValidationError);
      }
    },
  )?;

  // increment global aggregate ticket count
  TOTAL_TICKET_COUNT.update(storage, |n| -> ContractResult<_> {
    Ok(n + wager.ticket_count)
  })?;

  Ok((candidate, total_wager_size))
}

pub fn process_cancellation(storage: &mut dyn Storage) -> ContractResult<()> {
  Ok(STATUS.save(storage, &GameStatus::Canceled)?)
}

pub fn process_outcome_resolution(
  storage: &mut dyn Storage,
  candidate_indices: &Vec<u8>,
) -> ContractResult<()> {
  STATUS.save(storage, &GameStatus::Resolved)?;

  for candidate_index in candidate_indices.iter() {
    OUTCOME_INDICES.push_back(storage, candidate_index)?;
  }
  Ok(())
}

pub fn process_claim(
  storage: &mut dyn Storage,
  claimant: &Addr,
) -> ContractResult<Uint128> {
  // mark the account as claimed
  let account = ACCOUNTS.update(
    storage,
    claimant.clone(),
    |maybe_account| -> ContractResult<_> {
      if let Some(mut account) = maybe_account {
        if account.has_claimed {
          return Err(ContractError::NotAuthorized);
        }
        account.has_claimed = true;
        Ok(account)
      } else {
        Err(ContractError::NotAuthorized)
      }
    },
  )?;

  let status = load_game_status(storage)?;
  let ticket_price = TICKET_PRICE.load(storage)?;
  let total_ticket_count = TOTAL_TICKET_COUNT.load(storage)?;
  let total_winnings = ticket_price * Uint128::from(total_ticket_count);

  // compute claim amount differently depending on whether the user is claiming
  // a refund or their winnings.
  let claim_amount = match status {
    GameStatus::Canceled => ticket_price * Uint128::from(account.ticket_count),
    GameStatus::Resolved => {
      let mut claimant_winning_ticket_count = 0u32;
      let mut total_winning_ticket_count = 1u32;
      for result in OUTCOME_INDICES.iter(storage)? {
        let candidate_index = result.unwrap();
        let candidate = CANDIDATES.load(storage, candidate_index)?;
        total_winning_ticket_count *= candidate.ticket_count;
        claimant_winning_ticket_count += ACCOUNT_TICKET_COUNTS
          .load(storage, (claimant.clone(), candidate_index))
          .unwrap_or(0);
      }
      total_winnings.multiply_ratio(claimant_winning_ticket_count, total_winning_ticket_count)
    },
    _ => Uint128::zero(),
  };

  Ok(claim_amount)
}
