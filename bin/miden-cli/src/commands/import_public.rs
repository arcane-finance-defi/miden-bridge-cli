
use miden_client::{
    Client,
    account::AccountId,
};
use crate::{
    Parser, errors::CliError,
};

#[derive(Debug, Parser, Clone)]
#[clap(about = "Import public accounts")]
pub struct ImportPublicCmd {
    /// Public account id
    #[arg()]
    account_id: String,
}

impl ImportPublicCmd {
    pub async fn execute<AUTH>(&self, mut client: Client<AUTH>) -> Result<(), CliError> {
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
