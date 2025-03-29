mod cli;
mod commands;
mod config;
mod git;

use colored::Colorize;
use std::fs;
use std::path::Path;

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
                            "{} Repository name is required in non-interactive mode",
                            "ERROR:".red()
                        );
                        return;
                    }
                };
                let path = match sub_m.get_one::<String>("path") {
                    Some(path) => path,
                    None => {
                        eprintln!(
                            "{} Repository path is required in non-interactive mode",
                            "ERROR:".red()
                        );
                        return;
                    }
                };
                let remote = sub_m.get_one::<String>("remote").map(|s| s.as_str());
                let branch = sub_m.get_one::<String>("branch").map(|s| s.as_str());
                let groups = sub_m.get_one::<String>("groups").map(|s| s.as_str());

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
        _ => {
            println!("No command specified. Try 'gitpower --help' for more information.");
        }
    }
}
