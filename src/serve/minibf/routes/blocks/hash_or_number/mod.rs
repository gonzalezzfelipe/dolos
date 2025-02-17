use pallas::ledger::traverse::wellknown::GenesisValues;
use rocket::{get, http::Status, State};

use crate::{index::IndexStore, wal::redb::WalStore};

use super::Block;

pub mod addresses;
pub mod next;
pub mod previous;
pub mod txs;

#[get("/blocks/<hash_or_number>", rank = 2)]
pub fn route(
    hash_or_number: String,
    genesis: &State<GenesisValues>,
    wal: &State<WalStore>,
    index: &State<IndexStore>,
) -> Result<rocket::serde::json::Json<Block>, Status> {
    let block = Block::find_in_wal(wal, index, &hash_or_number, genesis)?;
    match block {
        Some(block) => Ok(rocket::serde::json::Json(block)),
        None => Err(Status::NotFound),
    }
}
