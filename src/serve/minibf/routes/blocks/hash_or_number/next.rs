use pallas::ledger::traverse::{wellknown::GenesisValues, MultiEraBlock};
use rocket::{get, http::Status, State};

use crate::{
    index::IndexStore,
    wal::{redb::WalStore, ReadUtils, WalReader},
};

use super::Block;

#[get("/blocks/<hash_or_number>/next", rank = 2)]
pub fn route(
    hash_or_number: String,
    _genesis: &State<GenesisValues>,
    wal: &State<WalStore>,
    index: &State<IndexStore>,
) -> Result<rocket::serde::json::Json<Block>, Status> {
    let possible_slots = index
        .get_possible_block_slots_by_block_hash(
            &hex::decode(hash_or_number.clone()).map_err(|_| Status::BadRequest)?,
        )
        .map_err(|_| Status::InternalServerError)?;

    let maybe_slot = wal
        .read_sparse_blocks_from_slots(&possible_slots)
        .map_err(|_| Status::InternalServerError)?
        .into_iter()
        .flatten()
        .filter_map(|raw| {
            if let Ok(block) = MultiEraBlock::decode(&raw.body) {
                if block.hash().to_string() == hash_or_number {
                    Some(raw.slot)
                } else {
                    None
                }
            } else {
                None
            }
        })
        .next();

    let Some(slot) = maybe_slot else {
        return Err(Status::NotFound);
    };

    let logseq = wal
        .locate_slot(&slot)
        .map_err(|_| Status::InternalServerError)?
        .unwrap(); // We already know the slot is in the WAL.

    let _iter = wal
        .crawl_from(Some(logseq))
        .map_err(|_| Status::InternalServerError)?
        .filter_apply()
        .into_blocks()
        .flatten();

    todo!()
}
