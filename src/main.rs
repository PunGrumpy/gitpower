mod cli;
mod commands;
mod config;
mod git;

use colored::Colorize;

fn main() {
    let matches = cli::build_cli().get_matches();

    let config_path = matches
        .get_one::<String>("config")
        .map(|s| s.as_str())
        .unwrap_or("~/.config/gitpower/config.yml");
    let config_path = shellexpand::tilde(config_path);

    let config = match config::load_config(&config_path) {
        Ok(config) => config,
        Err(e) => {
            eprintln!("{}: {}", "Error with config".red(), e);
            return;
        }
    };

    // Process commands
    match matches.subcommand() {
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
