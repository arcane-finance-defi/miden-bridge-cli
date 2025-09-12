use clap::Parser;
use miden_client::{Client, account::AccountId, Felt};
use miden_client::auth::TransactionAuthenticator;
use alloy_primitives::{Address, hex::FromHex};
use miden_bridge::notes::BRIDGE_USECASE;
use miden_objects::StarkField;
use miden_bridge::notes::crosschain::new_crosschain_note;
use miden_objects::note::NoteTag;
use miden_objects::transaction::OutputNote;
use miden_client::crypto::FeltRng;
use miden_client::transaction::TransactionRequestBuilder;
use crate::commands::new_transactions::execute_transaction;
use crate::errors::CliError;
use crate::utils::{bridge_note_tag, get_input_acc_id_by_prefix_or_default};

// ACCOUNT COMMAND
// ================================================================================================

/// Emits CROSSCHAIN note for funds transfer through the bridge
#[derive(Default, Debug, Clone, Parser)]
#[allow(clippy::option_option)]
pub struct CrosschainCmd {

    #[clap(short = 'c', long)]
    dest_chain: u32,

    #[clap(short = 'a', long = "dest-address")]
    dest_addr: String,

    #[clap(short = 'f', long)]
    asset_faucet_id: String,

    #[clap(short = 'm', long)]
    asset_amount: u64,

    /// Sender account.
    ///
    /// If no ID is provided it will display the current default account ID.
    /// If "none" is provided it will remove the default account else it will set the default
    /// account to the provided ID.
    #[clap(short, long, value_name = "ID")]
    sender: Option<String>,

    #[clap(short, long)]
    tag: Option<u32>,
}

impl CrosschainCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync + 'static>(&self, mut client: Client<AUTH>) -> Result<(), CliError> {
        let faucet_id = AccountId::from_hex(&self.asset_faucet_id)
            .map_err(|e| CliError::AccountId(e, "Malformed Faucet account id hex".to_string()))?;

        let evm_dest_address = Address::from_hex(self.dest_addr.as_str())
            .map_err(|_e| CliError::Input(format!("Non evm address hex {:?}", self.dest_addr)))?;

        let sender = get_input_acc_id_by_prefix_or_default(&client, self.sender.clone()).await?;

        let address_felts = [
            Felt::try_from(
                &evm_dest_address.0[..8]
            ).map_err(|e| CliError::Internal(Box::new(e)))?,
            Felt::try_from(
                &evm_dest_address.0[8..16]
            ).map_err(|e| CliError::Internal(Box::new(e)))?,
            Felt::from_bytes_with_padding(
                &evm_dest_address.0[16..20]
            )
        ];

        let dest_chain = Felt::from(self.dest_chain);

        let note = new_crosschain_note(
            client.rng().draw_word(),
            client.rng().draw_word(),
            dest_chain,
            address_felts,
            None,
            faucet_id,
            self.asset_amount,
            sender,
            self.tag.map(NoteTag::from).unwrap_or(bridge_note_tag())
        ).map_err(|e| CliError::Internal(Box::new(e)))?;

        let tx_request = TransactionRequestBuilder::new()
            .own_output_notes(vec![OutputNote::Full(note)])
            .build().map_err(|e| CliError::Internal(Box::new(e)))?;

        execute_transaction(
            &mut client,
            sender,
            tx_request,
            false,
            false
        ).await.map_err(|e| CliError::Internal(Box::new(e)))?;

        Ok(())
    }
}