use clap::{Parser, Subcommand, Command};
use clap_complete::{generate, Shell};
use std::io;
use synapse_cli::{CliError, handle_error};

#[derive(Parser)]
#[command(name = "synapse")]
#[command(about = "Synapse CLI - Fiat Gateway Command Line Interface")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Health check
    Health,

    /// Transaction commands
    #[command(subcommand)]
    Transactions(TransactionCommands),

    /// Settlement commands
    #[command(subcommand)]
    Settlements(SettlementCommands),

    /// Generate shell completions
    Completions {
        #[arg(value_name = "SHELL")]
        shell: String,
    },
}

#[derive(Subcommand)]
enum TransactionCommands {
    /// List transactions
    List,

    /// Search transactions
    Search,

    /// Get transaction details
    Get,
}

#[derive(Subcommand)]
enum SettlementCommands {
    /// List settlements
    List,

    /// Get settlement details
    Get,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Some(Commands::Health) => handle_health().await,
        Some(Commands::Transactions(tx_cmd)) => handle_transaction(tx_cmd).await,
        Some(Commands::Settlements(settlement_cmd)) => handle_settlement(settlement_cmd).await,
        Some(Commands::Completions { shell }) => handle_completions(&shell),
        None => handle_health().await,
    };

    if let Err(err) = result {
        let exit_code = handle_error(err);
        std::process::exit(exit_code);
    }
}

async fn handle_health() -> Result<(), CliError> {
    println!("✓ Health check passed");
    Ok(())
}

async fn handle_transaction(cmd: TransactionCommands) -> Result<(), CliError> {
    match cmd {
        TransactionCommands::List => {
            println!("Listing transactions...");
            Ok(())
        }
        TransactionCommands::Search => {
            println!("Searching transactions...");
            Ok(())
        }
        TransactionCommands::Get => {
            println!("Getting transaction...");
            Ok(())
        }
    }
}

async fn handle_settlement(cmd: SettlementCommands) -> Result<(), CliError> {
    match cmd {
        SettlementCommands::List => {
            println!("Listing settlements...");
            Ok(())
        }
        SettlementCommands::Get => {
            println!("Getting settlement...");
            Ok(())
        }
    }
}

fn handle_completions(shell: &str) -> Result<(), CliError> {
    let shell_enum = match shell {
        "bash" => Shell::Bash,
        "zsh" => Shell::Zsh,
        "fish" => Shell::Fish,
        _ => {
            return Err(CliError::Other(format!(
                "Unsupported shell: {}. Supported shells: bash, zsh, fish",
                shell
            )))
        }
    };

    let mut cmd = build_cli_command();
    generate(shell_enum, &mut cmd, "synapse", &mut io::stdout());
    Ok(())
}

fn build_cli_command() -> Command {
    Cli::command()
}
