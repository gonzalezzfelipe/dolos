use ::redb::{Database, Durability};
use pallas::ledger::traverse::MultiEraBlock;
use std::sync::Arc;

type Error = crate::index::IndexError;

use super::tables;
use crate::model::BlockSlot;

#[derive(Clone)]
pub struct IndexStore(pub Arc<Database>);

impl IndexStore {
    pub fn initialize(db: Database) -> Result<Self, Error> {
        let mut wx = db.begin_write()?;
        wx.set_durability(Durability::Immediate);

        tables::AddressApproxIndexTable::initialize(&wx)?;
        tables::BlockHashApproxIndexTable::initialize(&wx)?;
        tables::TxsApproxIndexTable::initialize(&wx)?;

        wx.commit()?;

        Ok(db.into())
    }

    pub(crate) fn db(&self) -> &Database {
        &self.0
    }

    pub(crate) fn db_mut(&mut self) -> Option<&mut Database> {
        Arc::get_mut(&mut self.0)
    }

    pub fn apply(&self, blocks: &[MultiEraBlock]) -> Result<(), Error> {
        let mut wx = self.db().begin_write()?;
        wx.set_durability(Durability::Eventual);

        for block in blocks {
            tables::AddressApproxIndexTable::apply(&wx, block)?;
            tables::BlockHashApproxIndexTable::apply(&wx, block)?;
            tables::TxsApproxIndexTable::apply(&wx, block)?;
        }

        wx.commit()?;

        Ok(())
    }

    pub fn undo(&self, block: &MultiEraBlock) -> Result<(), Error> {
        let mut wx = self.db().begin_write()?;
        wx.set_durability(Durability::Eventual);

        tables::AddressApproxIndexTable::undo(&wx, block)?;
        tables::BlockHashApproxIndexTable::undo(&wx, block)?;
        tables::TxsApproxIndexTable::undo(&wx, block)?;

        wx.commit()?;

        Ok(())
    }

    pub fn copy(&self, target: &Self) -> Result<(), Error> {
        let rx = self.db().begin_read()?;
        let wx = target.db().begin_write()?;

        tables::AddressApproxIndexTable::copy(&rx, &wx)?;
        tables::BlockHashApproxIndexTable::copy(&rx, &wx)?;
        tables::TxsApproxIndexTable::copy(&rx, &wx)?;

        wx.commit()?;

        Ok(())
    }

    pub fn finalize(&self, _: BlockSlot) -> Result<(), Error> {
        Ok(())
    }

    pub fn get_possible_block_slots_by_address(
        &self,
        address: &[u8],
    ) -> Result<Vec<BlockSlot>, Error> {
        let rx = self.db().begin_read()?;
        tables::AddressApproxIndexTable::get_by_address(&rx, address)
    }

    pub fn get_possible_block_slots_by_tx_hash(
        &self,
        tx_hash: &[u8],
    ) -> Result<Vec<BlockSlot>, Error> {
        let rx = self.db().begin_read()?;
        tables::TxsApproxIndexTable::get_by_tx_hash(&rx, tx_hash)
    }

    pub fn get_possible_block_slots_by_block_hash(
        &self,
        block_hash: &[u8],
    ) -> Result<Vec<BlockSlot>, Error> {
        let rx = self.db().begin_read()?;
        tables::BlockHashApproxIndexTable::get_by_block_hash(&rx, block_hash)
    }
}

impl From<Database> for IndexStore {
    fn from(value: Database) -> Self {
        Self(Arc::new(value))
    }
}
