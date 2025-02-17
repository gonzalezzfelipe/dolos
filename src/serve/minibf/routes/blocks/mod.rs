use pallas::ledger::traverse::{wellknown::GenesisValues, MultiEraBlock};
use rocket::http::Status;
use serde::{Deserialize, Serialize};

use crate::{
    index::IndexStore,
    wal::{redb::WalStore, ReadUtils, WalReader},
};

pub mod hash_or_number;
pub mod latest;
pub mod slot;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Block {
    pub slot: Option<u64>,
    pub hash: String,
    pub tx_count: u64,
    pub size: u64,
    pub time: u64,
    pub height: Option<u64>,
    pub epoch: Option<u64>,
    pub epoch_slot: Option<u64>,
    pub slot_leader: String,
    pub output: Option<String>,
    pub fees: Option<String>,
    pub block_vrf: Option<String>,
    pub op_cert: Option<String>,
    pub op_cert_counter: Option<String>,
    pub previous_block: Option<String>,
    pub next_block: Option<String>,
    pub confirmations: u64,
}

impl Block {
    pub fn find_in_wal(
        wal: &WalStore,
        index: &IndexStore,
        hash_or_number: &str,
        genesis: &GenesisValues,
    ) -> Result<Option<Block>, Status> {
        let possible_slots = index
            .get_possible_block_slots_by_block_hash(
                &hex::decode(hash_or_number).map_err(|_| Status::BadRequest)?,
            )
            .map_err(|_| Status::InternalServerError)?;

        let maybe_block = wal
            .read_sparse_blocks_from_slots(&possible_slots)
            .map_err(|_| Status::InternalServerError)?
            .into_iter()
            .flatten()
            .filter_map(|raw| {
                if let Ok(block) = MultiEraBlock::decode(&raw.body) {
                    if block.hash().to_string() == hash_or_number {
                        Some(raw.body)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .next();

        let Some(block_body) = maybe_block else {
            return Err(Status::NotFound);
        };
        let block = MultiEraBlock::decode(&block_body).unwrap(); // Safe
        let header = block.header();
        let prev = header.previous_hash().map(|h| h.to_string());
        let block_vrf = match header.vrf_vkey() {
            Some(v) => Some(
                bech32::encode::<bech32::Bech32>(bech32::Hrp::parse("vrf_vk").unwrap(), v)
                    .map_err(|_| Status::ServiceUnavailable)?,
            ),
            None => None,
        };
        let (epoch, epoch_slot) = block.epoch(genesis);

        let logseq = wal.assert_slot(&block.slot()).unwrap(); // Safe

        let mut confirmations = 0;
        let mut next = None;
        for raw in wal
            .crawl_from(Some(logseq))
            .map_err(|_| Status::InternalServerError)?
            .filter_apply()
            .into_blocks()
            .flatten()
        {
            // First is the same block.
            if confirmations > 0 && next.is_none() {
                next = Some(hex::encode(raw.hash))
            }
            confirmations += 1;
        }

        Ok(Some(Self {
            slot: Some(block.slot()),
            hash: block.hash().to_string(),
            tx_count: block.tx_count() as u64,
            size: block.body_size().unwrap_or(0) as u64,
            time: block.wallclock(genesis),
            epoch: Some(epoch),
            epoch_slot: Some(epoch_slot),
            height: Some(block.number()),
            previous_block: prev.clone(),
            block_vrf,
            confirmations,
            next_block: next,
            output: match block.tx_count() {
                0 => None,
                _ => Some(
                    block
                        .txs()
                        .iter()
                        .map(|tx| tx.outputs().iter().map(|o| o.value().coin()).sum::<u64>())
                        .sum::<u64>()
                        .to_string(),
                ),
            },
            fees: match block.tx_count() {
                0 => None,
                _ => Some(
                    block
                        .txs()
                        .iter()
                        .map(|tx| tx.fee().unwrap_or(0))
                        .sum::<u64>()
                        .to_string(),
                ),
            },
            ..Default::default()
        }))
    }
}
