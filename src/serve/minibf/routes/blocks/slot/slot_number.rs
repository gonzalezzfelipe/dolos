use pallas::ledger::traverse::wellknown::GenesisValues;
use rocket::{get, http::Status, State};

use crate::{
    index::IndexStore,
    serve::minibf::routes::blocks::Block,
    wal::{redb::WalStore, WalReader},
};

#[get("/blocks/slot/<slot_number>")]
pub fn route(
    slot_number: u64,
    genesis: &State<GenesisValues>,
    wal: &State<WalStore>,
    index: &State<IndexStore>,
) -> Result<rocket::serde::json::Json<Block>, Status> {
    let point = wal
        .read_block_from_slot(&slot_number)
        .map_err(|_| Status::InternalServerError)?;

    match point {
        Some(raw) => match Block::find_in_wal(wal, index, &raw.hash.to_string(), genesis) {
            Ok(Some(block)) => Ok(rocket::serde::json::Json(block)),
            Ok(None) => Err(Status::NotFound),
            Err(_) => Err(Status::ServiceUnavailable),
        },
        _ => Err(Status::NotFound),
    }
}
