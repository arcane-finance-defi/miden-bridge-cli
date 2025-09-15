use clap::Parser;
use miden_client::Client;
use miden_client::auth::TransactionAuthenticator;
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::config::CliEndpoint;
use crate::crosschain::reconstruct_crosschain_note;
use crate::errors::CliError;
use crate::notes::check_note_existence;
// MIX COMMAND
// ================================================================================================

/// Reconstructs the CROSSCHAIN note and pass it to the mixer operator
#[derive(Default, Debug, Clone, Parser)]
#[allow(clippy::option_option)]
pub struct MixCmd {
    #[clap(long)]
    dest_chain: u32,

    #[clap(long)]
    dest_address: String,

    /// BRIDGE note serial number hex
    #[clap(long)]
    bridge_serial_number: String,

    /// CROSSCHAIN serial number hex
    #[clap(short, long)]
    serial_number: String,

    /// CROSSCHAIN asset address hex
    #[clap(short, long)]
    faucet_id: String,

    /// CROSSCHAIN asset amount
    #[clap(short, long)]
    asset_amount: u64,
}

#[derive(Debug, Deserialize, Serialize)]
struct MixRequest {
    dest_chain_id: u64,
    dest_address: String,
    serial_num_hex: String,
    bridge_serial_num_hex: String,
    amount: u64,
    account_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct MixResponse {
    tx_id: String,
}

impl MixCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync + 'static>(
        &self,
        client: &mut Client<AUTH>,
        mixer_url: CliEndpoint,
    ) -> Result<(), CliError> {
        client.sync_state().await?;
        let (_, note_id) = reconstruct_crosschain_note(
            &self.serial_number,
            &self.bridge_serial_number,
            &self.dest_chain,
            &self.dest_address,
            &self.faucet_id,
            &self.asset_amount,
        )
        .await
        .map_err(|e| CliError::Internal(Box::new(e)))?;

        let note_id_hex = note_id.to_hex();
        info!("Reconstructed note id: {note_id_hex}");

        if check_note_existence(client, &note_id)
            .await
            .map_err(|e| CliError::Internal(Box::new(e)))?
        {
            debug!("Sending note: {note_id_hex} to mixer operator");

            let request = MixRequest {
                dest_chain_id: self.dest_chain.into(),
                dest_address: self.dest_address.clone(),
                serial_num_hex: self.serial_number.clone(),
                bridge_serial_num_hex: self.bridge_serial_number.clone(),
                amount: self.asset_amount,
                account_id: self.faucet_id.clone(),
            };

            let response = reqwest::Client::new()
                .post(format!("{}/api/v1/mix", mixer_url.to_string()))
                .json(&request)
                .send()
                .await
                .map_err(|e| CliError::Internal(Box::new(e)))?
                .json::<MixResponse>()
                .await
                .map_err(|e| CliError::Internal(Box::new(e)))?;

            println!("Generated tx id: {}", response.tx_id);

            Ok(())
        } else {
            Err(CliError::InvalidArgument(
                format!("Couldn't find a note {} onchain. Try later.", note_id_hex).to_string(),
            ))
        }
    }
}
