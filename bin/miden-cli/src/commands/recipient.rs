use clap::Parser;
use miden_lib::note::utils::build_p2id_recipient;
use miden_objects::crypto::utils::word_to_hex;
use miden_client::{
    Client,
    account::AccountId,
};
use miden_client::crypto::FeltRng;
use crate::errors::CliError;

// RECIPIENT COMMAND
// ================================================================================================

/// Generates RECIPIENT digest and serial number for P2ID note with specified receiver.
#[derive(Default, Debug, Clone, Parser)]
#[allow(clippy::option_option)]
pub struct RecipientCmd {
    /// P2ID receiver address hex.
    #[clap(short, long)]
    account_id: String
}

impl RecipientCmd {
    pub async fn execute(&self, mut client: Client) -> Result<(), CliError> {
        let rng = client.rng();
        let receiver = AccountId::from_hex(&self.account_id)
            .map_err(|e| CliError::AccountId(e, "Malformed Account id hex".to_string()))?;
        let serial_number = rng.draw_word();

        let recipient = build_p2id_recipient(receiver, serial_number.clone())
            .map_err(|e| CliError::Internal(Box::new(e)))?;

        let recipient_digest = recipient.digest().to_hex();
        let serial_number = word_to_hex(&serial_number)
            .map_err(|e| CliError::Internal(Box::new(e)))?;

        println!("Recipient: {recipient_digest}");
        println!("Serial number: 0x{serial_number}");

        Ok(())
    }
}