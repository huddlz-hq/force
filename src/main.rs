mod config;
mod env;
mod init;
mod runner;
mod state;
mod worktree;

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
    /// List active sessions
    Ls,
}

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Up { feature } => run_up(&feature),
        Commands::Down { feature } => run_down(&feature),
        Commands::Init => init::run_init(),
        Commands::Ls => run_ls(),
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

    // 2. Load configuration
    let force_config = config::load_config(&force_dir)?;

    // 3. Get project root (parent of .force/)
    let project_root = force_dir
        .parent()
        .ok_or("Invalid .force/ location")?;

    // 4. Create worktree
    let feature_slug = env::slugify(feature);
    let worktree_result = worktree::create_worktree(
        project_root,
        &feature_slug,
        &force_config.worktree.path,
    )?;

    if worktree_result.created {
        println!("Created worktree at: {}", worktree_result.path.display());
    } else {
        println!("Reusing worktree at: {}", worktree_result.path.display());
    }

    // 5. Generate environment
    let force_env = env::ForceEnv::new(feature, &force_dir, worktree_result.path);
    println!(
        "Feature: {} (slug: {})",
        force_env.feature, force_env.feature_slug
    );
    println!(
        "Port: {} (offset: {})",
        force_env.port, force_env.port_offset
    );

    // 6. Discover and load scripts
    let scripts = config::load_scripts(&force_dir)?;
    println!("Found {} script(s)", scripts.len());

    // 7. Execute scripts in order
    for script in scripts {
        runner::run_script(&script, &force_env)?;
    }

    // 8. Register session
    state::add_session(&force_dir, feature)?;

    println!("\nSession '{}' is ready!", feature);
    Ok(())
}

fn run_down(feature: &str) -> Result<(), Box<dyn std::error::Error>> {
    // 1. Find .force/ directory
    let force_dir = config::find_force_dir()?;
    println!("Found .force/ at: {}", force_dir.display());

    // 2. Load configuration
    let force_config = config::load_config(&force_dir)?;

    // 3. Get project root (parent of .force/)
    let project_root = force_dir
        .parent()
        .ok_or("Invalid .force/ location")?;

    // 4. Resolve worktree path
    let feature_slug = env::slugify(feature);
    let worktree_path = worktree::resolve_worktree_path(
        project_root,
        &feature_slug,
        &force_config.worktree.path,
    );

    // 5. Generate environment
    let force_env = env::ForceEnv::new(feature, &force_dir, worktree_path.clone());
    println!(
        "Feature: {} (slug: {})",
        force_env.feature, force_env.feature_slug
    );

    // 6. Discover and load scripts
    let scripts = config::load_scripts(&force_dir)?;
    println!("Found {} script(s)", scripts.len());

    // 7. Execute down scripts in reverse order (if worktree exists)
    if worktree_path.exists() {
        runner::run_down(&scripts, &force_env)?;
    } else {
        println!("Worktree not found, skipping down scripts");
    }

    // 8. Remove worktree if configured
    if force_config.worktree.remove_on_down {
        worktree::remove_worktree(project_root, &worktree_path)?;
        println!("Removed worktree at: {}", worktree_path.display());
    }

    // 9. Unregister session
    state::remove_session(&force_dir, feature)?;

    println!("\nSession '{}' torn down.", feature);
    Ok(())
}

fn run_ls() -> Result<(), Box<dyn std::error::Error>> {
    let force_dir = config::find_force_dir()?;
    let force_config = config::load_config(&force_dir)?;
    let sessions = state::list_sessions(&force_dir)?;

    if sessions.is_empty() {
        println!("No active sessions");
        return Ok(());
    }

    let project_root = force_dir
        .parent()
        .ok_or("Invalid .force/ location")?;

    println!("Active sessions:");
    for name in sessions {
        let feature_slug = env::slugify(&name);
        let worktree_path = worktree::resolve_worktree_path(
            project_root,
            &feature_slug,
            &force_config.worktree.path,
        );
        let force_env = env::ForceEnv::new(&name, &force_dir, worktree_path);
        println!("  {}  port {}", name, force_env.port);
    }
    Ok(())
}
