mod command;

use clap::{Parser, Subcommand};

/// Top-level CLI structure for the FreeCrime tool.
#[derive(Parser)]
#[command(name = "FreeCrime", about = "GTA Resource Tool", version = "0.1.0")]
struct Cli {
    /// The subcommand to execute.
    #[command(subcommand)]
    command: Commands,
}

/// Available commands for manipulating and viewing GTA resources.
#[derive(Subcommand)]
enum Commands {
    /// Show information about a game file.
    Info {
        /// Path to the game file (CMP, GRY, G24, FXT, FON, SDT, INI).
        file: String,
        /// Path to the CMP map file for style context.
        #[arg(long)]
        cmp: Option<String>,
    },
    /// Extract content from a game file into a directory.
    Extract {
        /// Path to the game file.
        file: String,
        /// Output directory or file.
        out: String,
        /// Path to the CMP map file for style transparency context.
        #[arg(long)]
        cmp: Option<String>,
    },
    /// Generate a top-down overview BMP of the map.
    Overview {
        /// Path to the CMP map file.
        cmp: String,
    },
    /// Interactive 3D map viewer using the Bevy engine.
    Display {
        /// Path to the CMP map file.
        cmp: String,
        /// Path to the GRY/G24 style file.
        gry: String,
        /// Initial camera position as a comma-separated triplet (X,Y,Z).
        #[arg(long)]
        camera_position: Option<String>,
        /// Initial camera rotation in degrees as a comma-separated triplet (YAW,PITCH,ROLL).
        #[arg(long)]
        camera_rotation: Option<String>,
    },
}

/// The main entry point of the FreeCrime utility.
fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Info { file, cmp } => {
            command::info::execute(&file, cmp.as_deref()).map_err(|e| anyhow::anyhow!("{}", e))?;
        }
        Commands::Extract { file, out, cmp } => {
            command::extract::execute(&file, &out, cmp.as_deref()).map_err(|e| anyhow::anyhow!("{}", e))?;
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

/// Parses a comma-separated string of three floats into an array of f32.
/// Returns None if the input string is None.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_vec3_valid() {
        assert_eq!(parse_vec3(Some("1.0, 2.5, -3.0".to_string())).unwrap(), Some([1.0, 2.5, -3.0]));
        assert_eq!(parse_vec3(None).unwrap(), None);
    }

    #[test]
    fn test_parse_vec3_invalid() {
        assert!(parse_vec3(Some("1.0, 2.0".to_string())).is_err());
        assert!(parse_vec3(Some("1.0, 2.0, a".to_string())).is_err());
    }
}
