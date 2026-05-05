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
        /// Initial camera position (X,Y,Z)
        #[arg(long)]
        camera_position: Option<String>,
        /// Initial camera rotation in degrees (YAW,PITCH,ROLL)
        #[arg(long)]
        camera_rotation: Option<String>,
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
        Commands::Display { cmp, gry, camera_position, camera_rotation } => {
            let pos = parse_vec3(camera_position)?;
            let rot = parse_vec3(camera_rotation)?;
            command::display::execute(&cmp, &gry, pos, rot).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
    }

    Ok(())
}

fn parse_vec3(s: Option<String>) -> anyhow::Result<Option<[f32; 3]>> {
    let Some(s) = s else { return Ok(None); };
    let parts: Vec<&str> = s.split(',').collect();
    if parts.len() != 3 {
        anyhow::bail!("Invalid vector format: expected X,Y,Z but got '{}'", s);
    }
    let x = parts[0].trim().parse::<f32>()?;
    let y = parts[1].trim().parse::<f32>()?;
    let z = parts[2].trim().parse::<f32>()?;
    Ok(Some([x, y, z]))
}
