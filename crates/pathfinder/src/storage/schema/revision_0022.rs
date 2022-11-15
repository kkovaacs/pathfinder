/// This migration adds the nonce column to the `contract_states` table for the starknet 0.10 update.
///
/// The column gets a default value of zero for old rows which is the correct value for versions pre 0.10.
pub(crate) fn migrate(tx: &rusqlite::Transaction<'_>) -> anyhow::Result<()> {
    use anyhow::Context;

    tracing::info!("Adding composite index to starknet_events table");

    tx.execute(
        r"CREATE INDEX starknet_events_from_address_block_number ON starknet_events(from_address, block_number)",
        [],
    )
    .context("Adding 'starknet_events_from_address_block_number' index")?;

    Ok(())
}
