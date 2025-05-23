use clap::Parser;
use miden_objects::note::{Note, NoteFile};
use miden_objects::utils::{Serializable, ToHex};
use miden_client::Client;
use miden_client::store::{NoteExportType, NoteFilter};
use crate::crosschain::reconstruct_crosschain_note;
use crate::errors::CliError;
use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt::format;
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
    asset_amount: u64
}


#[derive(Debug, Deserialize, Serialize)]
struct MixRequest {
    note_text: String,
    account_id: String,
}

#[derive(Debug, Deserialize, Serialize)]
struct MixResponse {
    tx_id: String,
}

impl MixCmd {
    pub async fn execute(&self, mut client: Client) -> Result<(), CliError> {
        client.sync_state().await?;
        let note_text = reconstruct_crosschain_note(
            &self.serial_number,
            &self.bridge_serial_number,
            &self.dest_chain,
            &self.dest_address,
            &self.faucet_id,
            &self.asset_amount
        ).await.map_err(|e| CliError::Internal(Box::new(e)))?;

        let note_id = client.import_note(note_text).await
            .map_err(|e| CliError::Internal(Box::new(e)))?;

        let note_id_hex = note_id.to_hex();
        println!("Reconstructed note id: {note_id_hex}");

        client.sync_state().await?;

        let input_note = client.get_input_notes(NoteFilter::Unique(note_id))
            .await?
            .pop()
            .unwrap();

        let inclusion_proof = match input_note.inclusion_proof() {
            Some(inclusion_proof) => Ok(inclusion_proof),
            _ => Err(CliError::InvalidArgument("Note still not commited".to_string()))
        }?;

        let note_text = NoteFile::NoteWithProof(
            Note::new(
                input_note.details().assets().clone(),
                input_note.metadata().unwrap().clone(),
                input_note.details().recipient().clone()
            ),
            inclusion_proof.clone()
        );

        let note_text = note_text.to_bytes().to_hex();

        let request = MixRequest {
            note_text,
            account_id: self.faucet_id.clone(),
        };

        let response = reqwest::Client::new()
            .post(format!("{}/mix", client.mixer_url().as_str()))
            .json(&request)
            .send()
            .await.map_err(|e| CliError::Internal(Box::new(e)))?
            .json::<MixResponse>()
            .await.map_err(|e| CliError::Internal(Box::new(e)))?;

        println!("Generated tx id: {}", response.tx_id);

        Ok(())
    }
}