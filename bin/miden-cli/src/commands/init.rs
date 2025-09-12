use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use clap::Parser;
use tracing::info;

use crate::CLIENT_CONFIG_FILE_NAME;
use crate::config::{CliConfig, CliEndpoint, Network};
use crate::errors::CliError;

use miden_client::consts::MIXER_DEFAULT_URL;

/// Contains the account component template file generated on build.rs, corresponding to the basic
/// wallet component.
const BASIC_WALLET_TEMPLATE_FILE: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/templates/", "basic-wallet.mct"));

/// Contains the account component template file generated on build.rs, corresponding to the
/// fungible faucet component.
const FAUCET_TEMPLATE_FILE: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/templates/", "basic-fungible-faucet.mct"));

/// Contains the account component template file generated on build.rs, corresponding to the basic
/// auth component.
const BASIC_AUTH_TEMPLATE_FILE: &[u8] =
    include_bytes!(concat!(env!("OUT_DIR"), "/templates/", "basic-auth.mct"));

// INIT COMMAND
// ================================================================================================

#[derive(Debug, Clone, Parser)]
#[command(
    about = "Initialize the client. It will create a file named `miden-client.toml` that holds \
the CLI and client configurations, and will be placed by default in the current working \
directory"
)]
pub struct InitCmd {
    /// Network configuration to use. Options are `devnet`, `testnet`, `localhost` or a custom RPC
    /// endpoint.
    #[clap(long, short)]
    network: Network,

    /// Path to the store file.
    #[arg(long)]
    store_path: Option<String>,

    /// RPC endpoint for the remote prover. Required if proving mode is set to remote.
    /// The endpoint must be in the form of "{protocol}://{hostname}:{port}", being the protocol
    /// and port optional.
    /// If the proving RPC isn't set, the proving mode will be set to local.
    #[arg(long)]
    remote_prover_endpoint: Option<String>,

    /// Maximum number of blocks the client can be behind the network.
    #[clap(long)]
    block_delta: Option<u32>,
}

impl InitCmd {
    pub fn execute(&self, config_file_path: &PathBuf) -> Result<(), CliError> {
        if config_file_path.exists() {
            return Err(CliError::Config(
                "Error with the configuration file".to_string().into(),
                format!(
                    "The file \"{CLIENT_CONFIG_FILE_NAME}\" already exists in the working directory. Please try using another directory or removing the file.",
                ),
            ));
        }

        let mut cli_config = CliConfig::default();

        let endpoint = CliEndpoint::try_from(self.network.clone())?;
        cli_config.rpc.endpoint = endpoint;

        if let Some(path) = &self.store_path {
            cli_config.store_filepath = PathBuf::from(path);
        }

        cli_config.remote_prover_endpoint = match &self.remote_prover_endpoint {
            Some(rpc) => CliEndpoint::try_from(rpc.as_str()).ok(),
            None => None,
        };

        cli_config.max_block_number_delta = self.block_delta;

        let config_as_toml_string = toml::to_string_pretty(&cli_config).map_err(|err| {
            CliError::Config("failed to serialize config".to_string().into(), err.to_string())
        })?;

        let mut file_handle = File::options()
            .write(true)
            .create_new(true)
            .open(config_file_path)
            .map_err(|err| {
            CliError::Config("failed to create config file".to_string().into(), err.to_string())
        })?;

        write_template_files(&cli_config)?;

        file_handle.write(config_as_toml_string.as_bytes()).map_err(|err| {
            CliError::Config("failed to write config file".to_string().into(), err.to_string())
        })?;

        cli_config.mixer_url = MIXER_DEFAULT_URL.try_into().unwrap();

        println!("Config file successfully created at: {}", config_file_path.display());

        Ok(())
    }
}

/// Creates the directory specified by `cli_config.component_template_directory`
/// and writes the default included component templates.
fn write_template_files(cli_config: &CliConfig) -> Result<(), CliError> {
    fs::create_dir_all(&cli_config.component_template_directory).map_err(|err| {
        CliError::Config(
            Box::new(err),
            "failed to create account component templates directory".into(),
        )
    })?;

    let wallet_template_path = cli_config.component_template_directory.join("basic-wallet.mct");
    let mut wallet_file = File::create(&wallet_template_path)?;
    wallet_file.write_all(BASIC_WALLET_TEMPLATE_FILE)?;

    // Write the faucet template file.
    // TODO: io errors should probably have their own context.
    let faucet_template_path =
        cli_config.component_template_directory.join("basic-fungible-faucet.mct");
    let mut faucet_file = File::create(&faucet_template_path)?;
    faucet_file.write_all(FAUCET_TEMPLATE_FILE)?;

    let basic_auth_template_path = cli_config.component_template_directory.join("basic-auth.mct");
    let mut basic_auth_file = File::create(&basic_auth_template_path)?;
    basic_auth_file.write_all(BASIC_AUTH_TEMPLATE_FILE)?;

    info!(
        "Template files successfully created in: {:?}",
        cli_config.component_template_directory
    );

    Ok(())
}
