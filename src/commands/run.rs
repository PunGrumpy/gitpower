use colored::*;
use std::process::Command as ProcessCommand;
use crate::config::{Config, get_repositories_by_names};

pub fn run_command(config: &Config, command: &str, names: &[&str]) {
    let repos = get_repositories_by_names(config, names);

    if repos.is_empty() {
        println!("No repositories to run command in.");
        return;
    }

    println!(
        "{} {}",
        "Running command in repositories:".green().bold(),
        command.yellow()
    );

    for repo in repos {
        println!("\n{} ({})", repo.name.yellow().bold(), repo.path);

        let path = shellexpand::tilde(&repo.path);
        let repo_path = std::path::Path::new(path.as_ref());

        if !repo_path.exists() {
            println!("  {} Repository path does not exist", "ERROR:".red().bold());
            continue;
        }

        // Run the custom command
        let output = ProcessCommand::new("sh")
            .current_dir(repo_path)
            .arg("-c")
            .arg(command)
            .output();

        match output {
            Ok(output) => {
                let stdout = String::from_utf8_lossy(&output.stdout);
                let stderr = String::from_utf8_lossy(&output.stderr);

                if !stdout.is_empty() {
                    println!("  {}", stdout);
                }

                if !stderr.is_empty() {
                    eprintln!("  {}", stderr);
                }

                if output.status.success() {
                    println!(
                        "  {} Command executed successfully",
                        "SUCCESS:".green().bold()
                    );
                } else {
                    println!(
                        "  {} Command failed with code {}",
                        "ERROR:".red().bold(),
                        output.status.code().unwrap_or(-1)
                    );
                }
            }
            Err(e) => {
                println!(
                    "  {} Failed to execute command: {}",
                    "ERROR:".red().bold(),
                    e
                );
            }
        }
    }

    println!("\n{}", "Command execution complete!".green().bold());
} 