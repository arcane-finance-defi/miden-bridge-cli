use clap::Parser;
use miden_objects::FieldElement;
use miden_objects::note::{Note, NoteExecutionHint, NoteFile, NoteMetadata, NoteTag, NoteType};
use miden_objects::utils::{Serializable, ToHex};
use miden_client::{Client, Felt};
use miden_client::store::{NoteExportType, NoteFilter};
use crate::crosschain::reconstruct_crosschain_note;
use crate::errors::CliError;
use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt::format;
use crate::errors::CliError::AccountId;
use crate::notes::check_note_existence;
use crate::utils::bridge_note_tag;
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
    pub async fn execute(&self, client: &mut Client) -> Result<(), CliError> {
        client.sync_state().await?;
        let (note_text, note_id) = reconstruct_crosschain_note(
            &self.serial_number,
            &self.bridge_serial_number,
            &self.dest_chain,
            &self.dest_address,
            &self.faucet_id,
            &self.asset_amount
        ).await.map_err(|e| CliError::Internal(Box::new(e)))?;

        let faucet_id = miden_objects::account::AccountId::from_hex(self.faucet_id.as_str())
            .map_err(|e| CliError::AccountId(e, "Malformed faucet id hex".to_string()))?;

        if check_note_existence(client, &note_id).await
            .map_err(|e| CliError::Internal(Box::new(e)))? {

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
                Some(inclusion_proof) => Ok(inclusion_proof.clone()),
                None => {
                    match client.get_note_inclusion_proof(note_id.clone()).await
                        .map_err(|err| CliError::Internal(Box::new(err)))? {
                        Some(proof) => Ok(proof),
                        _ => Err(CliError::InvalidArgument("Note still not commited".to_string()))
                    }
                }
            }?;

            let note_metadata: NoteMetadata = NoteMetadata::new(
                faucet_id.clone(),
                NoteType::Private,
                bridge_note_tag(),
                NoteExecutionHint::Always,
                Felt::ZERO
            ).map_err(|err| CliError::Internal(Box::new(err)))?;

            let note_text = NoteFile::NoteWithProof(
                Note::new(
                    input_note.details().assets().clone(),
                    NoteMetadata::new(
                        faucet_id,
                        NoteType::Private,
                        bridge_note_tag(),
                        NoteExecutionHint::Always,
                        Felt::ZERO
                    ).unwrap(),
                    input_note.details().recipient().clone()
                ),
                inclusion_proof
            );

            let note_text = note_text.to_bytes().to_hex();

            let request = MixRequest {
                dest_chain_id: self.dest_chain.into(),
                dest_address: self.dest_address.clone(),
                serial_num_hex: self.serial_number.clone(),
                bridge_serial_num_hex: self.bridge_serial_number.clone(),
                amount: self.asset_amount,
                account_id: self.faucet_id.clone(),
            };

            let response = reqwest::Client::new()
                .post(format!("{}/api/v1/mix", client.mixer_url().as_str()))
                .json(&request)
                .send()
                .await.map_err(|e| CliError::Internal(Box::new(e)))?
                .json::<MixResponse>()
                .await.map_err(|e| CliError::Internal(Box::new(e)))?;

            println!("Generated tx id: {}", response.tx_id);

            Ok(())
        } else {
            Err(CliError::InvalidArgument("Couldn't find a note onchain. Try later.".to_string()))
        }
    }
}