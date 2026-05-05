mod command;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "FreeCrime", about = "GTA Resource Tool", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Show information about a game file
    Info {
        /// Path to the game file (CMP, GRY, G24, FXT, FON, SDT, INI)
        file: String,
    },
    /// Extract content from a game file
    Extract {
        /// Path to the game file
        file: String,
        /// Output directory or file
        out: String,
    },
    /// Generate a top-down overview BMP of the map
    Overview {
        /// Path to the CMP map file
        cmp: String,
    },
    /// Interactive 3D map viewer (Bevy)
    Display {
        /// Path to the CMP map file
        cmp: String,
        /// Path to the GRY/G24 style file
        gry: String,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { file } => {
            command::info::execute(&file).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Commands::Extract { file, out } => {
            command::extract::execute(&file, &out).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Commands::Overview { cmp } => {
            command::overview::execute(&cmp).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Commands::Display { cmp, gry } => {
            command::display::execute(&cmp, &gry).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
    }

    Ok(())
}
