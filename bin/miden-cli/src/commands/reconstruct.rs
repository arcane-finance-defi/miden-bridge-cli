use clap::Parser;
use miden_lib::note::utils::build_p2id_recipient;
use miden_objects::asset::FungibleAsset;
use miden_objects::crypto::utils::word_to_hex;
use miden_objects::note::{Note, NoteAssets, NoteDetails, NoteExecutionHint, NoteExecutionMode, NoteFile, NoteMetadata, NoteTag, NoteType};
use miden_objects::utils::parse_hex_string_as_word;
use miden_client::{Client, Felt, account::AccountId, asset::Asset, ZERO};
use crate::errors::CliError;
use crate::notes::check_note_existence;
use crate::utils::bridge_note_tag;
// RECONSTRUCT COMMAND
// ================================================================================================

/// Reconstructs the P2ID note from the serial number, receiver and asset
#[derive(Default, Debug, Clone, Parser)]
#[allow(clippy::option_option)]
pub struct ReconstructCmd {
    /// P2ID receiver address hex
    #[clap(short = 'i', long)]
    account_id: String,

    /// P2ID serial number hex
    #[clap(short, long)]
    serial_number: String,

    /// P2ID asset address hex
    #[clap(short, long)]
    faucet_id: String,

    /// P2ID asset amount
    #[clap(short, long)]
    asset_amount: u64
}

impl ReconstructCmd {
    pub async fn execute(&self, client: &mut Client) -> Result<(), CliError> {
        let receiver = AccountId::from_hex(&self.account_id)
            .map_err(|e| CliError::AccountId(e, "Malformed Account id hex".to_string()))?;
        let faucet_id = AccountId::from_hex(&self.faucet_id)
            .map_err(|e| CliError::AccountId(e, "Malformed faucet id hex".to_string()))?;
        let serial_number = parse_hex_string_as_word(&self.serial_number)
            .map_err(|_| CliError::InvalidArgument("serial-number".to_string()))?;

        let recipient = build_p2id_recipient(receiver, serial_number.clone())
            .map_err(|e| CliError::Internal(Box::new(e)))?;

        let note_details = NoteDetails::new(
            NoteAssets::new(vec![
                FungibleAsset::new(
                    faucet_id, self.asset_amount
                ).map_err(|e| CliError::Internal(Box::new(e)))?.into()
            ]).map_err(|e| CliError::Internal(Box::new(e)))?,
            recipient,
        );

        let note_tag = bridge_note_tag();

        let note_id = note_details.id();

        let note_id_hex = note_id.to_hex();
        println!("Reconstructed note id: {note_id_hex}");

        if check_note_existence(client, &note_id).await
            .map_err(|e| CliError::Internal(Box::new(e)))? {

            let note_text = NoteFile::NoteDetails {
                details: note_details,
                after_block_num: 0.into(),
                tag: Some(note_tag),
            };

            client.import_note(note_text).await
                .map_err(|e| CliError::Internal(Box::new(e)))?;
            Ok(())
        } else {
            Err(CliError::InvalidArgument("Couldn't find a note onchain. Try later.".to_string()))
        }
    }
}