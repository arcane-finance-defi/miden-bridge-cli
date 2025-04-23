use clap::Parser;
use miden_bridge::notes::bridge::croschain;
use miden_lib::note::utils::build_p2id_recipient;
use miden_objects::crypto::utils::word_to_hex;
use miden_objects::FieldElement;
use miden_objects::note::{NoteInputs, NoteRecipient};
use miden_client::{Client, account::AccountId, Felt};
use miden_client::crypto::FeltRng;
use crate::errors::CliError;
use crate::crosschain::{build_crosschain_recipient, evm_address_to_felts};
use std::fmt::Display;
use clap::ValueEnum;
// RECIPIENT COMMAND
// ================================================================================================

#[derive(ValueEnum, Debug, Clone)]
enum RecipientType {
    P2ID,
    CROSSCHAIN
}

impl Default for RecipientType {
    fn default() -> Self {
        RecipientType::P2ID
    }
}

/// Generates RECIPIENT digest and serial number for P2ID note with specified receiver.
#[derive(Default, Debug, Clone, Parser)]
#[allow(clippy::option_option)]
pub struct RecipientCmd {
    /// P2ID receiver address hex.
    #[clap(short, long)]
    account_id: Option<String>,

    #[clap(short, long, value_enum, default_value_t = RecipientType::P2ID)]
    note_type: RecipientType,

    #[clap(long)]
    dest_chain: Option<u32>,

    #[clap(long)]
    dest_address: Option<String>,
}

impl RecipientCmd {
    pub async fn execute(&self, mut client: Client) -> Result<(), CliError> {
        let rng = client.rng();
        let receiver = AccountId::from_hex(&self.account_id)
            .map_err(|e| CliError::AccountId(e, "Malformed Account id hex".to_string()))?;
        let serial_number = rng.draw_word();
        let serial_number_hex = word_to_hex(&serial_number)
            .map_err(|e| CliError::Internal(Box::new(e)))?;

        let recipient_digest = match &self {
            Self {
                note_type: RecipientType::P2ID,
                account_id: Some(account_id),
                ..
            } => {
                let receiver = AccountId::from_hex(account_id)
                    .map_err(|e| CliError::AccountId(e, "Malformed Account id hex".to_string()))?;

                let recipient = build_p2id_recipient(receiver, serial_number)
                    .map_err(|e| CliError::Internal(Box::new(e)))?;

                Ok(recipient.digest().to_hex())
            },
            Self {
                note_type: RecipientType::CROSSCHAIN,
                dest_chain: Some(dest_chain),
                dest_address: Some(dest_address),
                ..
            } => {
                let bridge_note_serial_number = rng.draw_word();
                let bridge_note_serial_number_hex = word_to_hex(&bridge_note_serial_number)
                    .map_err(|e| CliError::Internal(Box::new(e)))?;

                let dest_addr = evm_address_to_felts(dest_address.to_string())
                    .map_err(|e| CliError::Internal(Box::new(e)))?;

                let recipient = build_crosschain_recipient(
                    serial_number,
                    bridge_note_serial_number,
                    *dest_chain,
                    dest_addr
                ).map_err(|e| CliError::Internal(Box::new(e)))?;

                println!("BRIDGE serial number: 0x{bridge_note_serial_number_hex}");
                Ok(recipient.digest().to_hex())
            },
            _ => {
                Err(CliError::Input("Wrong arguments set".to_string()))
            }
        }?;

        println!("Recipient: {recipient_digest}");
        println!("Serial number: 0x{serial_number_hex}");

        Ok(())
    }
}