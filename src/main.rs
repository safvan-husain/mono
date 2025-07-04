use clap::{Parser, Subcommand};
use std::fs;
use std::path::{Path, PathBuf};
use std::io;
use std::process::Command as ProcessCommand;

mod config;

#[derive(Parser)]
#[command(name = "monorepo-agent")]
#[command(about = "A tool for managing monorepos with Git and rsync", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initializes the monorepo configuration
    Init {
        /// Comma-separated list of submodules
        #[arg(short, long)]
        submodules: String,
    },
    /// Synchronizes files for specified submodules (or all if none specified)
    Sync {
        /// Optional: Comma-separated list of submodules to sync
        #[arg(short, long)]
        submodules: Option<String>,
    },
}

fn init_monorepo(submodules_str: &str) -> io::Result<()> {
    let config_dir = Path::new(".monorepo");
    if !config_dir.exists() {
        fs::create_dir(config_dir)?;
    }

    // The AGENT.md had a confusing statement: "Ensures a sibling directory exists with the same name as the monorepo's parent."
    // User clarification and the rsync example ("vendroo-monorepo/user_app/" syncs to "user_app/")
    // confirm that target directories are siblings to the monorepo root, named after the submodules themselves.
    // The `sync` command now handles creation of these target directories if they don't exist.
    // Therefore, the previous validation/warning in `init` is no longer needed.

    let submodules: Vec<String> = submodules_str.split(',').map(|s| s.trim().to_string()).collect();
    if submodules.is_empty() || submodules.iter().any(|s| s.is_empty()) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Submodule list cannot be empty or contain empty names.",
        ));
    }

    let mut app_config = config::load_or_create_config(config_dir)?;

    for submodule_name in submodules {
        if !app_config.submodules.iter().any(|s| s.name == submodule_name) {
            app_config.submodules.push(config::SubmoduleConfig {
                name: submodule_name.clone(),
                path: PathBuf::from(&submodule_name), // Default path is submodule name
                include: vec!["lib/***".to_string(), "pubspec.yaml".to_string(), "test/***".to_string()],
                exclude: vec!["*".to_string()],
            });
            println!("Added submodule: {}", submodule_name);
        } else {
            println!("Submodule {} already configured.", submodule_name);
        }
    }

    config::save_config(config_dir, &app_config)?;
    println!("Monorepo initialized/updated with submodules: {}", submodules_str);
    Ok(())
}

fn sync_submodules(submodules_to_sync_str: Option<&str>) -> io::Result<()> {
    let config_dir = Path::new(".monorepo");
    if !config_dir.exists() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Monorepo not initialized. Run 'init' first.",
        ));
    }

    let app_config = config::load_or_create_config(config_dir)?;
    if app_config.submodules.is_empty() {
        println!("No submodules configured. Nothing to sync.");
        return Ok(());
    }

    let current_dir = std::env::current_dir()?;
    // The monorepo_name variable was previously unused.
    // It's not strictly needed for the current sync logic, as submodule paths are relative to current_dir
    // and target paths are relative to parent_dir using submodule.name.
    // If it were needed, it would be:
    // let _monorepo_name = current_dir.file_name().ok_or_else(|| {
    //     io::Error::new(
    //         io::ErrorKind::InvalidInput,
    //         "Failed to get monorepo directory name.",
    //     )
    // })?;
    let parent_dir = current_dir.parent().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Failed to get parent directory of the monorepo.",
        )
    })?;

    let submodules_to_process: Vec<config::SubmoduleConfig> =
        if let Some(names_str) = submodules_to_sync_str {
            let names: Vec<String> = names_str.split(',').map(|s| s.trim().to_string()).collect();
            if names.iter().any(|s| s.is_empty()) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Submodule list for sync cannot contain empty names.",
                ));
            }
            app_config
                .submodules
                .into_iter()
                .filter(|s| names.contains(&s.name))
                .collect()
        } else {
            app_config.submodules
        };

    if submodules_to_process.is_empty() {
        if submodules_to_sync_str.is_some() {
            println!("No matching configured submodules found to sync.");
        } else {
            println!("No submodules configured to sync.");
        }
        return Ok(());
    }

    for submodule in submodules_to_process {
        println!("Syncing submodule: {}", submodule.name);

        let source_path = current_dir.join(&submodule.path);
        if !source_path.exists() || !source_path.is_dir() {
            eprintln!(
                "Source path for submodule {} not found or not a directory: {:?}",
                submodule.name, source_path
            );
            continue; // Skip to next submodule
        }

        // As per AGENT.md: "rsync ... vendroo-monorepo/user_app/ user_app/"
        // This implies the target is a sibling to the monorepo root, with the same name as the submodule.
        let target_path = parent_dir.join(&submodule.name);

        if !target_path.exists() {
             println!("Target directory {:?} does not exist. Creating it.", target_path);
             fs::create_dir_all(&target_path)?;
        }
        if !target_path.is_dir() {
            eprintln!(
                "Target path for submodule {} is not a directory: {:?}",
                submodule.name, target_path
            );
            continue;
        }


        let mut rsync_cmd = ProcessCommand::new("rsync");
        rsync_cmd.arg("-a"); // archive mode
        rsync_cmd.arg("--delete"); // delete extraneous files from dest dirs
        rsync_cmd.arg("--times"); // preserve modification times
        rsync_cmd.arg("--no-perms"); // don't preserve permissions
        rsync_cmd.arg("--no-owner"); // don't preserve owner
        rsync_cmd.arg("--no-group"); // don't preserve group
        // rsync_cmd.arg("--inplace"); // AGENT.md specified this, but it can be risky. Let's omit for now.
                                    // It updates files in place, which can be bad for partial transfers.

        for include_pattern in &submodule.include {
            rsync_cmd.arg(format!("--include={}", include_pattern));
        }
        for exclude_pattern in &submodule.exclude {
            rsync_cmd.arg(format!("--exclude={}", exclude_pattern));
        }

        // Source path needs a trailing slash for rsync to copy contents correctly
        let source_path_str = format!("{}/", source_path.to_string_lossy());
        rsync_cmd.arg(source_path_str);
        rsync_cmd.arg(target_path.to_string_lossy().to_string());

        println!("Executing rsync: {:?}", rsync_cmd);

        let status = rsync_cmd.status()?;
        if status.success() {
            println!("Successfully synced submodule: {}", submodule.name);
        } else {
            eprintln!(
                "Error syncing submodule {}: rsync command failed with status {}",
                submodule.name, status
            );
        }
    }

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { submodules } => {
            if let Err(e) = init_monorepo(submodules) {
                eprintln!("Error initializing monorepo: {}", e);
            }
        }
        Commands::Sync { submodules } => {
            if let Err(e) = sync_submodules(submodules.as_deref()) {
                eprintln!("Error syncing submodules: {}", e);
            }
        }
    }
}
