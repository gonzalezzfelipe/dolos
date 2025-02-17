use pallas::ledger::traverse::MultiEraBlock;
use thiserror::Error;

use crate::ledger::BrokenInvariant;
use crate::model::BlockSlot;

pub mod redb;

#[derive(Debug, Error)]
pub enum IndexError {
    #[error("broken invariant")]
    BrokenInvariant(#[source] BrokenInvariant),

    #[error("storage error")]
    StorageError(#[source] ::redb::Error),

    #[error("address decoding error")]
    AddressDecoding(pallas::ledger::addresses::Error),

    #[error("query not supported")]
    QueryNotSupported,

    #[error("invalid store version")]
    InvalidStoreVersion,

    #[error("decoding error")]
    DecodingError(#[source] pallas::codec::minicbor::decode::Error),
}

impl From<::redb::TableError> for IndexError {
    fn from(value: ::redb::TableError) -> Self {
        Self::StorageError(value.into())
    }
}

impl From<::redb::CommitError> for IndexError {
    fn from(value: ::redb::CommitError) -> Self {
        Self::StorageError(value.into())
    }
}

impl From<::redb::StorageError> for IndexError {
    fn from(value: ::redb::StorageError) -> Self {
        Self::StorageError(value.into())
    }
}

impl From<::redb::TransactionError> for IndexError {
    fn from(value: ::redb::TransactionError) -> Self {
        Self::StorageError(value.into())
    }
}

impl From<pallas::ledger::addresses::Error> for IndexError {
    fn from(value: pallas::ledger::addresses::Error) -> Self {
        Self::AddressDecoding(value)
    }
}

/// A persistent store for ledger state
#[derive(Clone)]
#[non_exhaustive]
pub enum IndexStore {
    Redb(redb::IndexStore),
}

impl IndexStore {
    pub fn get_possible_block_slots_by_address(
        &self,
        address: &[u8],
    ) -> Result<Vec<BlockSlot>, IndexError> {
        match self {
            IndexStore::Redb(x) => x.get_possible_block_slots_by_address(address),
        }
    }

    pub fn get_possible_block_slots_by_tx_hash(
        &self,
        tx_hash: &[u8],
    ) -> Result<Vec<BlockSlot>, IndexError> {
        match self {
            IndexStore::Redb(x) => x.get_possible_block_slots_by_tx_hash(tx_hash),
        }
    }

    pub fn get_possible_block_slots_by_block_hash(
        &self,
        block_hash: &[u8],
    ) -> Result<Vec<BlockSlot>, IndexError> {
        match self {
            IndexStore::Redb(x) => x.get_possible_block_slots_by_block_hash(block_hash),
        }
    }

    pub fn apply(&self, blocks: &[MultiEraBlock]) -> Result<(), IndexError> {
        match self {
            IndexStore::Redb(x) => x.apply(blocks),
        }
    }

    pub fn undo(&self, block: &MultiEraBlock) -> Result<(), IndexError> {
        match self {
            IndexStore::Redb(x) => x.undo(block),
        }
    }

    pub fn finalize(&self, until: BlockSlot) -> Result<(), IndexError> {
        match self {
            IndexStore::Redb(x) => x.finalize(until),
        }
    }

    pub fn copy(&self, target: &Self) -> Result<(), IndexError> {
        match (self, target) {
            (Self::Redb(x), Self::Redb(target)) => x.copy(target),
        }
    }
}

impl From<redb::IndexStore> for IndexStore {
    fn from(value: redb::IndexStore) -> Self {
        Self::Redb(value)
    }
}
