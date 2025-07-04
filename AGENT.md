Monorepo Management Agent
Project Overview
This Rust-based project provides a command-line tool for managing a monorepo, integrating with Git for version control and rsync for efficient file synchronization. The tool initializes a monorepo configuration, manages submodules, and synchronizes specified files or directories (e.g., lib/, pubspec.yaml, test/) to a sibling directory with the same name as the monorepo's parent directory.
Purpose
The agent simplifies monorepo management by:

Initializing a monorepo with a configuration file or folder (similar to .git).

Requiring a sibling directory to the monorepo parent for synchronization.

Supporting submodule management within the monorepo.

Using rsync to sync specific files/folders to the sibling directory, e.g.:
rsync -a --delete --times --no-perms --no-owner --no-group --inplace --include='lib/***' --include='pubspec.yaml' --include='test/***' --exclude='*' vendroo-monorepo/user_app/ user_app/



Project Setup
The project is in its initial phase and will be developed in Rust. Below is the planned structure and initial setup.
Directory Structure
monorepo-agent/
├── .git/                    # Git repository for the project
├── src/                     # Rust source code
│   ├── main.rs              # Main entry point
│   └── config.rs            # Configuration management module
├── .monorepo/               # Monorepo configuration folder (created by the binary)
├── Cargo.toml               # Rust project configuration
└── Agent.md                 # This file

Key Features

Initialization:

Command: monorepo-agent init <submodule>
Creates a .monorepo folder with configuration details.
Ensures a sibling directory exists with the same name as the monorepo's parent.
Stores submodule information in the configuration.


Synchronization:

Uses rsync to sync specific files/folders (e.g., lib/, pubspec.yaml, test/) to the sibling directory.
Example: For a monorepo at vendroo-monorepo/user_app/, syncs to user_app/.


Submodule Management:

Supports adding, removing, and updating submodules within the monorepo.
Integrates with Git for version control.



Initial Rust Setup
The project will be initialized as a Rust binary project using Cargo.
Cargo.toml
[package]
name = "monorepo-agent"
version = "0.1.0"
edition = "2021"

[dependencies]
clap = { version = "4.5", features = ["derive"] }  # CLI argument parsing
serde = { version = "1.0", features = ["derive"] } # Serialization for config
serde_json = "1.0"                                # JSON handling for config

main.rs (Initial Skeleton)
use clap::{Parser, Subcommand};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(name = "monorepo-agent")]
#[command(about = "A tool for managing monorepos with Git and rsync", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Init {
        #[arg(short, long)]
        submodule: String,
    },
}

fn init_monorepo(submodule: &str) -> std::io::Result<()> {
    let config_dir = ".monorepo";
    if !Path::new(config_dir).exists() {
        fs::create_dir(config_dir)?;
        println!("Initialized monorepo with submodule: {}", submodule);
    } else {
        println!("Monorepo already initialized.");
    }
    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { submodule } => {
            if let Err(e) = init_monorepo(submodule) {
                eprintln!("Error initializing monorepo: {}", e);
            }
        }
    }
}

Next Steps

Implement configuration file handling in .monorepo (e.g., JSON format for submodules and sync rules).
Add rsync integration for file synchronization to the sibling directory.
Implement Git integration for submodule management.
Add validation to ensure the sibling directory exists and matches the monorepo's parent name.

Usage Example

Initialize the monorepo, coma seprated sub modules:
monorepo-agent init user_app, business_app, backend


Creates .monorepo and validates user_app/ sibling directory.


Sync files to the sibling directory:
rsync -a --delete --times --no-perms --no-owner --no-group --inplace --include='lib/***' --include='pubspec.yaml' --include='test/***' --exclude='*' vendroo-monorepo/user_app/ user_app/


the include, exclude should be configurable to each submodule.


Development Notes

The project is in its early stages, focusing on Rust for performance and reliability.
Future iterations will include error handling, logging, and advanced rsync configurations.
Collaboration with coding agents will streamline development and testing.
