use crate::params::Params;
use clap::{Args, Subcommand};

/// CLI commands for querying and updating wasm module parameters.
#[derive(Args, Debug, Clone)]
pub struct WasmParamsCli {
    #[command(subcommand)]
    pub command: WasmParamsCommand,
}

/// Supported subcommands under `wasm params`.
#[derive(Subcommand, Debug, Clone)]
pub enum WasmParamsCommand {
    /// Print the current parameters in JSON form.
    Show,
    /// Update the parameters from a JSON file.
    Set(SetParamsArgs),
}

/// Arguments for the `set` subcommand.
#[derive(Args, Debug, Clone)]
pub struct SetParamsArgs {
    /// Path to a JSON file containing a `Params` object.
    pub file: std::path::PathBuf,
}

/// Execute the selected parameters command using the provided keeper.
///
/// The actual interaction with a node context is left for later
/// integration. This function currently just parses the file and
/// returns the new parameters so that tests can verify parsing.
pub fn run_params_command(cli: WasmParamsCli) -> anyhow::Result<Option<Params>> {
    match cli.command {
        WasmParamsCommand::Show => {
            // Display logic will be wired once query handlers are implemented.
            println!("TODO: query and display current wasm params");
            Ok(None)
        }
        WasmParamsCommand::Set(args) => {
            let contents = std::fs::read_to_string(&args.file)?;
            let params: Params = serde_json::from_str(&contents)?;
            // State update will be hooked up to governance/keeper in future work.
            Ok(Some(params))
        }
    }
}
