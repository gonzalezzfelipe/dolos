pub mod txs;

use pallas::ledger::traverse::wellknown::GenesisValues;
use rocket::{get, http::Status, State};

use crate::{
    index::IndexStore,
    serve::minibf::routes::blocks::Block,
    wal::{redb::WalStore, WalReader},
};

#[get("/blocks/latest", rank = 1)]
pub fn route(
    genesis: &State<GenesisValues>,
    wal: &State<WalStore>,
    index: &State<IndexStore>,
) -> Result<rocket::serde::json::Json<Block>, Status> {
    let tip = wal.find_tip().map_err(|_| Status::ServiceUnavailable)?;
    match tip {
        None => Err(Status::ServiceUnavailable),
        Some((_, point)) => {
            let raw = wal
                .read_block(&point)
                .map_err(|_| Status::ServiceUnavailable)?;

            match Block::find_in_wal(wal, index, &raw.hash.to_string(), genesis) {
                Ok(Some(block)) => Ok(rocket::serde::json::Json(block)),
                Ok(None) => Err(Status::NotFound),
                Err(_) => Err(Status::ServiceUnavailable),
            }
        }
    }
}
