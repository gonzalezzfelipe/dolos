use dolos::wal::{self, RawBlock, ReadUtils, WalReader as _};
use itertools::Itertools;
use miette::{Context, IntoDiagnostic};
use pallas::ledger::traverse::MultiEraBlock;

use crate::feedback::Feedback;

#[derive(Debug, clap::Args)]
pub struct Args;

pub fn run(config: &crate::Config, _args: &Args, feedback: &Feedback) -> miette::Result<()> {
    //crate::common::setup_tracing(&config.logging)?;

    let progress = feedback.slot_progress_bar();
    progress.set_message("rebuilding index");

    let wal = crate::common::open_wal(config).context("opening WAL store")?;
    let index = dolos::index::redb::IndexStore::in_memory_v1()
        .into_diagnostic()
        .context("creating in-memory index store")?;

    let index = dolos::index::IndexStore::Redb(index);

    let (_, tip) = wal
        .find_tip()
        .into_diagnostic()
        .context("finding WAL tip")?
        .ok_or(miette::miette!("no WAL tip found"))?;

    match tip {
        wal::ChainPoint::Origin => progress.set_length(0),
        wal::ChainPoint::Specific(slot, _) => progress.set_length(slot),
    }

    let remaining = wal
        .crawl_from(None)
        .into_diagnostic()
        .context("crawling wal")?
        .filter_forward()
        .into_blocks()
        .flatten();

    for chunk in remaining.chunks(100).into_iter() {
        let bodies = chunk.map(|RawBlock { body, .. }| body).collect_vec();

        let blocks: Vec<_> = bodies
            .iter()
            .map(|b| MultiEraBlock::decode(b))
            .try_collect()
            .into_diagnostic()
            .context("decoding blocks")?;

        index
            .apply(&blocks)
            .into_diagnostic()
            .context("apply block batch deltas")?;

        blocks.last().inspect(|b| progress.set_position(b.slot()));
    }

    let ledger_path = crate::common::define_index_path(config).context("finding index path")?;

    let disk = dolos::index::redb::IndexStore::open(ledger_path, None)
        .into_diagnostic()
        .context("opening index db")?;

    let disk = dolos::index::IndexStore::Redb(disk);

    let pb = feedback.indeterminate_progress_bar();
    pb.set_message("copying memory index into disc");

    index
        .copy(&disk)
        .into_diagnostic()
        .context("copying from memory db into disc")?;

    pb.abandon_with_message("index copy to disk finished");

    Ok(())
}
