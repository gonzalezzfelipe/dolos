use ::redb::{ReadTransaction, ReadableTable as _};
use ::redb::{TableDefinition, WriteTransaction};
use pallas::ledger::addresses::Address;
use pallas::ledger::traverse::MultiEraBlock;
use std::hash::{DefaultHasher, Hash as _, Hasher};

use crate::model::BlockSlot;

type Error = crate::index::IndexError;

pub struct AddressApproxIndexTable;
impl AddressApproxIndexTable {
    pub const DEF: TableDefinition<'static, u64, Vec<u64>> =
        TableDefinition::new("addressapproxindex");

    pub fn initialize(wx: &WriteTransaction) -> Result<(), Error> {
        wx.open_table(Self::DEF)?;

        Ok(())
    }

    pub fn compute_key(address: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        address.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_by_address(rx: &ReadTransaction, address: &[u8]) -> Result<Vec<BlockSlot>, Error> {
        let table = rx.open_table(Self::DEF)?;
        let default = Ok(vec![]);
        let key = Self::compute_key(address);
        match table.get(key)? {
            Some(value) => Ok(value.value().clone()),
            None => default,
        }
    }

    pub fn apply(wx: &WriteTransaction, block: &MultiEraBlock) -> Result<(), Error> {
        let mut table = wx.open_table(Self::DEF)?;

        let addresses = block
            .txs()
            .iter()
            .flat_map(|tx| {
                tx.produces()
                    .into_iter()
                    .map(|(_, meo)| match meo.address().map_err(Error::from)? {
                        Address::Shelley(x) => Ok(x.to_vec()),
                        Address::Stake(x) => Ok(x.to_vec()),
                        Address::Byron(x) => Ok(x.to_vec()),
                    })
                    .collect::<Vec<Result<Vec<u8>, Error>>>()
            })
            .collect::<Result<Vec<Vec<u8>>, Error>>()?;

        for address in addresses {
            let key = Self::compute_key(&address);
            let slot = block.slot();

            let maybe_new = match table.get(key)? {
                Some(value) => {
                    let mut previous = value.value().clone();
                    if !previous.contains(&slot) {
                        previous.push(slot);
                        Some(previous)
                    } else {
                        None
                    }
                }
                None => Some(vec![slot]),
            };
            if let Some(new) = maybe_new {
                table.insert(key, new)?;
            }
        }

        Ok(())
    }

    pub fn undo(wx: &WriteTransaction, block: &MultiEraBlock) -> Result<(), Error> {
        let mut table = wx.open_table(Self::DEF)?;

        let addresses = block
            .txs()
            .iter()
            .flat_map(|tx| {
                tx.produces()
                    .into_iter()
                    .map(|(_, meo)| match meo.address().map_err(Error::from)? {
                        Address::Shelley(x) => Ok(x.to_vec()),
                        Address::Stake(x) => Ok(x.to_vec()),
                        Address::Byron(x) => Ok(x.to_vec()),
                    })
                    .collect::<Vec<Result<Vec<u8>, Error>>>()
            })
            .collect::<Result<Vec<Vec<u8>>, Error>>()?;

        for address in addresses {
            let key = Self::compute_key(&address);
            let slot = block.slot();

            let maybe_new = match table.get(key)? {
                Some(value) => {
                    let mut previous = value.value().clone();
                    match previous.iter().position(|x| *x == slot) {
                        Some(index) => {
                            previous.remove(index);
                            Some(previous)
                        }
                        None => None,
                    }
                }
                None => None,
            };
            if let Some(new) = maybe_new {
                table.insert(key, new)?;
            }
        }

        Ok(())
    }

    pub fn copy(rx: &ReadTransaction, wx: &WriteTransaction) -> Result<(), Error> {
        let source = rx.open_table(Self::DEF)?;
        let mut target = wx.open_table(Self::DEF)?;

        for entry in source.iter()? {
            let (k, v) = entry?;
            target.insert(k.value(), v.value())?;
        }

        Ok(())
    }
}

pub struct TxsApproxIndexTable;
impl TxsApproxIndexTable {
    pub const DEF: TableDefinition<'static, u64, Vec<u64>> = TableDefinition::new("txsapproxindex");

    pub fn initialize(wx: &WriteTransaction) -> Result<(), Error> {
        wx.open_table(Self::DEF)?;

        Ok(())
    }

    pub fn compute_key(tx_hash: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        tx_hash.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_by_tx_hash(rx: &ReadTransaction, tx_hash: &[u8]) -> Result<Vec<BlockSlot>, Error> {
        let table = rx.open_table(Self::DEF)?;
        let default = Ok(vec![]);
        let key = Self::compute_key(tx_hash);
        match table.get(key)? {
            Some(value) => Ok(value.value().clone()),
            None => default,
        }
    }

    pub fn apply(wx: &WriteTransaction, block: &MultiEraBlock) -> Result<(), Error> {
        let mut table = wx.open_table(Self::DEF)?;

        let tx_hashes = block
            .txs()
            .iter()
            .map(|tx| tx.hash().to_vec())
            .collect::<Vec<Vec<u8>>>();

        for tx_hash in tx_hashes {
            let key = Self::compute_key(&tx_hash);
            let slot = block.slot();

            let maybe_new = match table.get(key)? {
                Some(value) => {
                    let mut previous = value.value().clone();
                    if !previous.contains(&slot) {
                        previous.push(slot);
                        Some(previous)
                    } else {
                        None
                    }
                }
                None => Some(vec![slot]),
            };
            if let Some(new) = maybe_new {
                table.insert(key, new)?;
            }
        }

        Ok(())
    }

    pub fn undo(wx: &WriteTransaction, block: &MultiEraBlock) -> Result<(), Error> {
        let mut table = wx.open_table(Self::DEF)?;

        let tx_hashes = block
            .txs()
            .iter()
            .map(|tx| tx.hash().to_vec())
            .collect::<Vec<Vec<u8>>>();

        for tx_hash in tx_hashes {
            let key = Self::compute_key(&tx_hash);
            let slot = block.slot();

            let maybe_new = match table.get(key)? {
                Some(value) => {
                    let mut previous = value.value().clone();
                    match previous.iter().position(|x| *x == slot) {
                        Some(index) => {
                            previous.remove(index);
                            Some(previous)
                        }
                        None => None,
                    }
                }
                None => None,
            };
            if let Some(new) = maybe_new {
                table.insert(key, new)?;
            }
        }

        Ok(())
    }

    pub fn copy(rx: &ReadTransaction, wx: &WriteTransaction) -> Result<(), Error> {
        let source = rx.open_table(Self::DEF)?;
        let mut target = wx.open_table(Self::DEF)?;

        for entry in source.iter()? {
            let (k, v) = entry?;
            target.insert(k.value(), v.value())?;
        }

        Ok(())
    }
}

pub struct BlockHashApproxIndexTable;
impl BlockHashApproxIndexTable {
    pub const DEF: TableDefinition<'static, u64, Vec<u64>> =
        TableDefinition::new("blockhashapproxindex");

    pub fn initialize(wx: &WriteTransaction) -> Result<(), Error> {
        wx.open_table(Self::DEF)?;

        Ok(())
    }

    pub fn compute_key(block_hash: &[u8]) -> u64 {
        let mut hasher = DefaultHasher::new();
        block_hash.hash(&mut hasher);
        hasher.finish()
    }

    pub fn get_by_block_hash(
        rx: &ReadTransaction,
        block_hash: &[u8],
    ) -> Result<Vec<BlockSlot>, Error> {
        let table = rx.open_table(Self::DEF)?;
        let default = Ok(vec![]);
        let key = Self::compute_key(block_hash);
        match table.get(key)? {
            Some(value) => Ok(value.value().clone()),
            None => default,
        }
    }

    pub fn apply(wx: &WriteTransaction, block: &MultiEraBlock) -> Result<(), Error> {
        let mut table = wx.open_table(Self::DEF)?;

        let key = Self::compute_key(block.hash().as_ref());
        let slot = block.slot();

        let maybe_new = match table.get(key)? {
            Some(value) => {
                let mut previous = value.value().clone();
                if !previous.contains(&slot) {
                    previous.push(slot);
                    Some(previous)
                } else {
                    None
                }
            }
            None => Some(vec![slot]),
        };
        if let Some(new) = maybe_new {
            table.insert(key, new)?;
        }

        Ok(())
    }

    pub fn undo(wx: &WriteTransaction, block: &MultiEraBlock) -> Result<(), Error> {
        let mut table = wx.open_table(Self::DEF)?;

        let key = Self::compute_key(block.hash().as_ref());
        let slot = block.slot();

        let maybe_new = match table.get(key)? {
            Some(value) => {
                let mut previous = value.value().clone();
                match previous.iter().position(|x| *x == slot) {
                    Some(index) => {
                        previous.remove(index);
                        Some(previous)
                    }
                    None => None,
                }
            }
            None => None,
        };
        if let Some(new) = maybe_new {
            table.insert(key, new)?;
        }

        Ok(())
    }

    pub fn copy(rx: &ReadTransaction, wx: &WriteTransaction) -> Result<(), Error> {
        let source = rx.open_table(Self::DEF)?;
        let mut target = wx.open_table(Self::DEF)?;

        for entry in source.iter()? {
            let (k, v) = entry?;
            target.insert(k.value(), v.value())?;
        }

        Ok(())
    }
}
