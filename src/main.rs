mod config;
mod env;
mod init;
mod runner;

use clap::{Parser, Subcommand};
use std::process;

#[derive(Parser)]
#[command(name = "force")]
#[command(about = "A force multiplier for parallel AI development")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Spin up a new session (alias: u)
    #[command(alias = "u")]
    Up {
        /// Feature name for the session
        feature: String,
    },
    /// Tear down a session (alias: d)
    #[command(alias = "d")]
    Down {
        /// Feature name for the session
        feature: String,
    },
    /// Initialize a .force/ directory with example scripts
    Init,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Up { feature } => run_up(&feature),
        Commands::Down { feature } => run_down(&feature),
        Commands::Init => init::run_init(),
    };

    if let Err(e) = result {
        eprintln!("Error: {}", e);
        process::exit(1);
    }
}

fn run_up(feature: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Find .force/ directory
    let force_dir = config::find_force_dir()?;
    println!("Found .force/ at: {}", force_dir.display());

    // 2. Generate environment
    let force_env = env::ForceEnv::new(feature, &force_dir);
    println!("Feature: {} (slug: {})", force_env.feature, force_env.feature_slug);
    println!("Port: {} (offset: {})", force_env.port, force_env.port_offset);

    // 3. Discover and load scripts
    let scripts = config::load_scripts(&force_dir)?;
    println!("Found {} script(s)", scripts.len());

    // 4. Execute scripts in order
    for script in scripts {
        runner::run_script(&script, &force_env)?;
    }

    println!("\nSession '{}' is ready!", feature);
    Ok(())
}

fn run_down(feature: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Find .force/ directory
    let force_dir = config::find_force_dir()?;
    println!("Found .force/ at: {}", force_dir.display());

    // 2. Generate environment
    let force_env = env::ForceEnv::new(feature, &force_dir);
    println!("Feature: {} (slug: {})", force_env.feature, force_env.feature_slug);

    // 3. Discover and load scripts
    let scripts = config::load_scripts(&force_dir)?;
    println!("Found {} script(s)", scripts.len());

    // 4. Execute down scripts in reverse order
    runner::run_down(&scripts, &force_env)?;

    println!("\nSession '{}' torn down.", feature);
    Ok(())
}
