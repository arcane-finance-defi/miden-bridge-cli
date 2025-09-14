use miden_client::account::{AccountFile, AccountId};
use miden_client::auth::TransactionAuthenticator;
use miden_client::keystore::KeyStoreError;
use miden_client::utils::Deserializable;
use miden_client::{Client, ClientError};

use crate::errors::CliError;
use crate::{CliKeyStore, Parser};

#[derive(Debug, Parser, Clone)]
#[clap(about = "Import public accounts")]
pub struct ImportPublicCmd {
    /// Public account id
    #[arg()]
    account_id: String,
}

impl ImportPublicCmd {
    pub async fn execute<AUTH: TransactionAuthenticator + Sync>(
        &self,
        mut client: Client<AUTH>,
        keystore: CliKeyStore,
    ) -> Result<(), CliError> {
        let account_id = AccountId::from_hex(self.account_id.as_str())
            .map_err(|e| CliError::AccountId(e, "Malformed Account id hex".to_string()))?;

        if account_id.is_public() {
            client.import_account_by_id(account_id).await?;
            Ok(())
        } else {
            Err(CliError::Input("Non-public account ID is passed to import".to_string()))?
        }
    }
}

// IMPORT ACCOUNT
// ================================================================================================

async fn import_account<AUTH: TransactionAuthenticator + Sync>(
    client: &mut Client<AUTH>,
    keystore: &CliKeyStore,
    account_data_file_contents: &[u8],
    overwrite: bool,
) -> Result<AccountId, CliError> {
    let account_data = AccountFile::read_from_bytes(account_data_file_contents)
        .map_err(ClientError::DataDeserializationError)?;
    let account_id = account_data.account.id();

    account_data
        .auth_secret_keys
        .iter()
        .map(|key| keystore.add_key(key))
        .collect::<Result<Vec<()>, KeyStoreError>>()
        .map_err(CliError::KeyStore)?;

    client
        .add_account(&account_data.account, account_data.account_seed, overwrite)
        .await?;

    Ok(account_id)
}
