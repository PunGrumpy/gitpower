mod cli;
mod commands;
mod config;
mod git;

use colored::Colorize;
use std::fs;
use std::path::Path;
use config::Repository;

fn main() {
    let cli = cli::build_cli();
    let matches = cli.clone().get_matches();

    // Handle completion command first
    if let Some(("completion", sub_m)) = matches.subcommand() {
        let shell = sub_m.get_one::<String>("shell").unwrap();
        match shell.as_str() {
            "bash" => cli::print_completion(clap_complete::Shell::Bash),
            "zsh" => cli::print_completion(clap_complete::Shell::Zsh),
            "fish" => cli::print_completion(clap_complete::Shell::Fish),
            "powershell" => cli::print_completion(clap_complete::Shell::PowerShell),
            _ => unreachable!(),
        }
        return;
    }

    let config_path = matches
        .get_one::<String>("config")
        .map(|s| s.as_str())
        .unwrap_or("~/.config/gitpower/config.yml");
    let config_path = shellexpand::tilde(config_path);
    let config_path = Path::new(config_path.as_ref());

    let mut config = match config::load_config(config_path.to_str().unwrap()) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}: {}", "Error with config".red(), e);
            return;
        }
    };

    // Process commands
    match matches.subcommand() {
        Some(("init", sub_m)) => {
            let non_interactive = sub_m.get_flag("non-interactive");

            if non_interactive {
                // Handle non-interactive mode
                let name = match sub_m.get_one::<String>("name") {
                    Some(name) => name,
                    None => {
                        eprintln!(
                            "{} Repository name is required in non-interactive mode. Use --name <name>",
                            "ERROR:".red()
                        );
                        return;
                    }
                };
                let path = match sub_m.get_one::<String>("path") {
                    Some(path) => path,
                    None => {
                        eprintln!(
                            "{} Repository path is required in non-interactive mode. Use --path <path>",
                            "ERROR:".red()
                        );
                        return;
                    }
                };

                // Validate repository path
                let expanded_path = shellexpand::tilde(path);
                let repo_path = Path::new(expanded_path.as_ref());
                
                if !repo_path.exists() {
                    if let Err(e) = fs::create_dir_all(repo_path) {
                        eprintln!(
                            "{} Failed to create repository directory '{}': {}",
                            "ERROR:".red(),
                            repo_path.display(),
                            e
                        );
                        return;
                    }
                }

                let remote = sub_m.get_one::<String>("remote").map(|s| s.as_str());
                let branch = sub_m.get_one::<String>("branch").map(|s| s.as_str());
                let groups = sub_m.get_one::<String>("groups").map(|s| s.as_str());

                // Create repository and validate it
                let repo = Repository {
                    name: name.clone(),
                    path: path.clone(),
                    remote: remote.map(|s| s.to_string()),
                    branch: branch.map(|s| s.to_string()),
                    groups: groups.map(|s| s.split(',').map(|s| s.trim().to_string()).collect()),
                };

                if let Err(e) = repo.validate() {
                    eprintln!("{} {}", "ERROR:".red(), e);
                    return;
                }

                commands::init_repository(&mut config, name, path, remote, branch, groups);
            } else {
                // Handle interactive mode
                commands::init_repository_interactive(&mut config);
            }

            // Save updated config
            if let Ok(yaml) = serde_yaml::to_string(&config) {
                if let Err(e) = fs::write(config_path, yaml) {
                    eprintln!("{} Failed to save config: {}", "ERROR:".red(), e);
                }
            }
        }
        Some(("list", _)) => commands::list_repositories(&config),
        Some(("status", sub_m)) => {
            let repo_names: Vec<&str> = if let Some(values) = sub_m.get_many::<String>("repos") {
                values.map(|s| s.as_str()).collect()
            } else {
                vec![] // Empty means all repositories
            };
            commands::show_repository_status(&config, &repo_names);
        }
        Some(("sync", sub_m)) => {
            let repo_names: Vec<&str> = if let Some(values) = sub_m.get_many::<String>("repos") {
                values.map(|s| s.as_str()).collect()
            } else {
                vec![] // Empty means all repositories
            };
            commands::sync_repositories(&config, &repo_names);
        }
        Some(("pull", sub_m)) => {
            let repo_names: Vec<&str> = if let Some(values) = sub_m.get_many::<String>("repos") {
                values.map(|s| s.as_str()).collect()
            } else {
                vec![] // Empty means all repositories
            };
            commands::pull_repositories(&config, &repo_names);
        }
        Some(("run", sub_m)) => {
            let command = sub_m.get_one::<String>("command").unwrap();
            let repo_names: Vec<&str> = if let Some(values) = sub_m.get_many::<String>("repos") {
                values.map(|s| s.as_str()).collect()
            } else {
                vec![] // Empty means all repositories
            };
            commands::run_command(&config, command, &repo_names);
        }
        Some(("interactive", _)) => {
            let mut app = commands::App::new(config);
            if let Err(e) = app.run() {
                eprintln!("{}: {}", "Error in interactive mode".red(), e);
            }
        }
        _ => {
            println!("No command specified. Try 'gitpower --help' for more information.");
        }
    }
}
