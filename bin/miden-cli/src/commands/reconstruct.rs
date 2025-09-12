use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use clap::{Parser, ValueEnum};
use miden_lib::note::utils::build_p2id_recipient;
use miden_objects::asset::FungibleAsset;
use miden_objects::{Felt, FieldElement};
use miden_objects::note::{Note, NoteAssets, NoteDetails, NoteExecutionHint, NoteFile, NoteMetadata, NoteType};
use miden_objects::utils::{parse_hex_string_as_word, Serializable};
use tracing::{info, warn};
use miden_client::Client;
use miden_client::auth::TransactionAuthenticator;
use crate::crosschain::reconstruct_crosschain_note;
use crate::errors::CliError;
use crate::notes::{check_note_existence, get_fetched_note_proof};
use crate::utils::{bridge_note_tag, parse_account_id};
// RECONSTRUCT COMMAND
// ================================================================================================


#[derive(ValueEnum, Debug, Clone)]
enum ReconstructType {
    P2ID,
    CROSSCHAIN
}

impl Default for ReconstructType {
    fn default() -> Self {
        ReconstructType::P2ID
    }
}

/// Reconstructs the P2ID note from the serial number, receiver and asset
#[derive(Default, Debug, Clone, Parser)]
#[allow(clippy::option_option)]
pub struct ReconstructCmd {
    // Note type to reconstruction
    #[clap(value_enum, short, long, default_value_t = ReconstructType::P2ID)]
    note_type: ReconstructType,

    #[clap(long)]
    dest_chain: Option<u32>,

    #[clap(long)]
    dest_address: Option<String>,

    #[clap(long)]
    bridge_serial_number: Option<String>,

    /// P2ID receiver address hex
    #[clap(short = 'i', long)]
    account_id: Option<String>,

    /// P2ID serial number hex
    #[clap(short, long)]
    serial_number: String,

    /// P2ID asset address hex
    #[clap(short, long)]
    faucet_id: Option<String>,

    /// P2ID asset amount
    #[clap(short, long)]
    asset_amount: Option<u64>,

    /// Should the resulting note be exported as file
    #[clap(long)]
    export: Option<bool>,

    /// Desired filename for the binary file. Defaults to the note ID if not provided.
    #[arg(short, long)]
    filename: Option<PathBuf>,
}

impl ReconstructCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync>(&self, client: &mut Client<AUTH>) -> Result<(), CliError> {
        client.sync_state().await?;
        let (note_text, note_id) = match self {
            Self {
                note_type: ReconstructType::P2ID,
                account_id: Some(account_id),
                serial_number,
                faucet_id: Some(faucet_id),
                asset_amount: Some(asset_amount),
                ..
            } => {
                let receiver = parse_account_id(&client, account_id).await?;
                let faucet_id = parse_account_id(&client, faucet_id).await?;
                let serial_number = parse_hex_string_as_word(serial_number)
                    .map_err(|_| CliError::InvalidArgument("serial-number".to_string()))?;

                let recipient = build_p2id_recipient(receiver, serial_number.into())
                    .map_err(|e| CliError::Internal(Box::new(e)))?;

                let note_details = NoteDetails::new(
                    NoteAssets::new(vec![
                        FungibleAsset::new(
                            faucet_id, *asset_amount
                        ).map_err(|e| CliError::Internal(Box::new(e)))?.into()
                    ]).map_err(|e| CliError::Internal(Box::new(e)))?,
                    recipient,
                );

                let note_tag = bridge_note_tag();

                let note_id = note_details.id();

                let note_id_hex = note_id.to_hex();
                println!("Reconstructed note id: {note_id_hex}");

                Ok((NoteFile::NoteDetails {
                    details: note_details,
                    after_block_num: 0.into(),
                    tag: Some(note_tag)
                }, note_id))
            }
            Self {
                note_type: ReconstructType::CROSSCHAIN,
                serial_number,
                bridge_serial_number: Some(bridge_serial_number),
                dest_address: Some(dest_address),
                dest_chain: Some(dest_chain),
                faucet_id: Some(faucet_id),
                asset_amount: Some(asset_amount),
                ..
            } => {
                reconstruct_crosschain_note(
                    serial_number,
                    bridge_serial_number,
                    dest_chain,
                    dest_address,
                    faucet_id,
                    asset_amount,
                ).await.map_err(|e| CliError::Internal(Box::new(e)))
            },
            _ => Err(CliError::Input("Wrong arguments set".to_string()))
        }?;

        if check_note_existence(client, &note_id).await
            .map_err(|e| CliError::Internal(Box::new(e)))? {


            if let Some(true) = self.export {
                let (note_details, tag) = match note_text {
                    NoteFile::NoteDetails { details, tag: Some(tag), .. } => (details, tag),
                    _ => unreachable!()
                };

                let proof = get_fetched_note_proof(note_id).await?.expect("note proof from RPC");

                let note_text = NoteFile::NoteWithProof(
                    Note::new(
                        note_details.assets().clone(),
                        NoteMetadata::new(
                            parse_account_id(client, self.faucet_id.clone().unwrap().as_str()).await?,
                            NoteType::Private,
                            tag,
                            NoteExecutionHint::Always,
                            Felt::ZERO
                        ).map_err(|err| CliError::Internal(Box::new(err)))?,
                        note_details.recipient().clone()
                    ),
                    proof
                );

                let file_path = if let Some(filename) = &self.filename {
                    filename.clone()
                } else {
                    let current_dir = std::env::current_dir()?;
                    current_dir.join(format!("{}.mno", note_id))
                };

                info!("Writing file to {}", file_path.to_string_lossy());
                let mut file = File::create(file_path)?;
                file.write_all(&note_text.to_bytes()).map_err(CliError::IO)?;

                println!("Successfully exported note {note_id}");

            } else {
                client.import_note(note_text).await
                    .map_err(|e| CliError::Internal(Box::new(e)))?;

                info!("Note {} successfully imported", note_id.to_hex());
            }
        } else {
            warn!("Note {} was not found", note_id.to_hex());
        }

        Ok(())
    }
}