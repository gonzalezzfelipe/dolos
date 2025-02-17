use ::redb::{Database, MultimapTableHandle as _, TableHandle as _};
use itertools::Itertools;
use log::info;
use pallas::ledger::traverse::MultiEraBlock;
use std::path::Path;

use tracing::{debug, warn};

mod tables;
mod v1;

use super::*;

const DEFAULT_CACHE_SIZE_MB: usize = 500;

fn compute_schema_hash(db: &Database) -> Result<Option<String>, IndexError> {
    let mut hasher = pallas::crypto::hash::Hasher::<160>::new();

    let rx = db
        .begin_read()
        .map_err(|e| IndexError::StorageError(e.into()))?;

    let names_1 = rx
        .list_tables()
        .map_err(|e| IndexError::StorageError(e.into()))?
        .map(|t| t.name().to_owned());

    let names_2 = rx
        .list_multimap_tables()
        .map_err(|e| IndexError::StorageError(e.into()))?
        .map(|t| t.name().to_owned());

    let mut names = names_1.chain(names_2).collect_vec();

    debug!(tables = ?names, "tables names used to compute hash");

    if names.is_empty() {
        // this db hasn't been initialized, we can't compute hash
        return Ok(None);
    }

    // sort to make sure we don't depend on some redb implementation regarding order
    // of the tables.
    names.sort();

    names.into_iter().for_each(|n| hasher.input(n.as_bytes()));

    let hash = hasher.finalize();

    Ok(Some(hash.to_string()))
}

fn open_db(path: impl AsRef<Path>, cache_size: Option<usize>) -> Result<Database, IndexError> {
    let db = Database::builder()
        .set_repair_callback(|x| warn!(progress = x.progress() * 100f64, "ledger db is repairing"))
        .set_cache_size(1024 * 1024 * cache_size.unwrap_or(DEFAULT_CACHE_SIZE_MB))
        .create(path)
        .map_err(|x| IndexError::StorageError(x.into()))?;

    Ok(db)
}

impl From<::redb::Error> for IndexError {
    fn from(value: ::redb::Error) -> Self {
        IndexError::StorageError(value)
    }
}

const V1_HASH: &str = "a6bfcdd302d6a5c4cd57c8d643c4ae657d18adf2";

#[derive(Clone)]
pub enum IndexStore {
    SchemaV1(v1::IndexStore),
}

impl IndexStore {
    pub fn open(path: impl AsRef<Path>, cache_size: Option<usize>) -> Result<Self, IndexError> {
        let db = open_db(path, cache_size)?;
        let hash = compute_schema_hash(&db)?;

        let schema = match hash.as_deref() {
            // use stable schema if no hash
            None => {
                info!("no state db schema, initializing as v1");
                v1::IndexStore::initialize(db)?.into()
            }
            Some(V1_HASH) => {
                info!("detected state db schema v1");
                v1::IndexStore::from(db).into()
            }
            Some(x) => panic!("can't recognize db hash {}", x),
        };

        Ok(schema)
    }

    pub fn in_memory_v1() -> Result<Self, IndexError> {
        let db = ::redb::Database::builder()
            .create_with_backend(::redb::backends::InMemoryBackend::new())
            .unwrap();

        let store = v1::IndexStore::initialize(db)?;
        Ok(store.into())
    }

    pub fn db(&self) -> &Database {
        match self {
            IndexStore::SchemaV1(x) => x.db(),
        }
    }

    pub fn db_mut(&mut self) -> Option<&mut Database> {
        match self {
            IndexStore::SchemaV1(x) => x.db_mut(),
        }
    }

    pub fn get_possible_block_slots_by_address(
        &self,
        address: &[u8],
    ) -> Result<Vec<BlockSlot>, IndexError> {
        match self {
            IndexStore::SchemaV1(x) => Ok(x.get_possible_block_slots_by_address(address)?),
        }
    }

    pub fn get_possible_block_slots_by_tx_hash(
        &self,
        tx_hash: &[u8],
    ) -> Result<Vec<BlockSlot>, IndexError> {
        match self {
            IndexStore::SchemaV1(x) => Ok(x.get_possible_block_slots_by_tx_hash(tx_hash)?),
        }
    }

    pub fn get_possible_block_slots_by_block_hash(
        &self,
        block_hash: &[u8],
    ) -> Result<Vec<BlockSlot>, IndexError> {
        match self {
            IndexStore::SchemaV1(x) => Ok(x.get_possible_block_slots_by_block_hash(block_hash)?),
        }
    }

    pub fn apply(&self, blocks: &[MultiEraBlock]) -> Result<(), IndexError> {
        match self {
            IndexStore::SchemaV1(x) => Ok(x.apply(blocks)?),
        }
    }

    pub fn undo(&self, block: &MultiEraBlock) -> Result<(), IndexError> {
        match self {
            IndexStore::SchemaV1(x) => Ok(x.undo(block)?),
        }
    }

    pub fn finalize(&self, until: BlockSlot) -> Result<(), IndexError> {
        match self {
            IndexStore::SchemaV1(x) => Ok(x.finalize(until)?),
        }
    }

    pub fn copy(&self, target: &Self) -> Result<(), IndexError> {
        match (self, target) {
            (IndexStore::SchemaV1(x), IndexStore::SchemaV1(target)) => Ok(x.copy(target)?),
        }
    }
}

impl From<v1::IndexStore> for IndexStore {
    fn from(value: v1::IndexStore) -> Self {
        Self::SchemaV1(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_hash_computation() {
        let store = IndexStore::in_memory_v1().unwrap();
        let hash = compute_schema_hash(store.db()).unwrap();
        assert_eq!(hash.unwrap(), V1_HASH);
    }
}
