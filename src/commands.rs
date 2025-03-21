use colored::*;
use std::path::Path;
use std::process::Command as ProcessCommand;

use crate::config::Config;
use crate::config::get_repositories_by_names;
use crate::git::{run_git_command, run_git_command_with_output};

pub fn list_repositories(config: &Config) {
    println!("{}", "Configured Repositories:".green().bold());
    for repo in &config.repositories {
        let path = shellexpand::tilde(&repo.path);
        let repo_path = Path::new(path.as_ref());

        let status_indicator = if repo_path.exists() {
            "✓".green()
        } else {
            "✗".red()
        };

        println!(
            "  {} {} - {}",
            status_indicator,
            repo.name.yellow(),
            repo.path
        );
        if let Some(branch) = &repo.branch {
            println!("    Branch: {}", branch);
        }
        if let Some(groups) = &repo.groups {
            println!("    Groups: {}", groups.join(", "));
        }
    }

    if let Some(groups) = &config.groups {
        println!("\n{}", "Configured Groups:".green().bold());
        for group in groups {
            println!(
                "  {} - {} repositories",
                group.name.yellow(),
                group.repositories.len()
            );
            println!("    Repos: {}", group.repositories.join(", "));
        }
    }
}

pub fn show_repository_status(config: &Config, names: &[&str]) {
    let repos = get_repositories_by_names(config, names);

    if repos.is_empty() {
        println!("No repositories to check status.");
        return;
    }

    println!("{}", "Repository Status:".green().bold());

    for repo in repos {
        let path = shellexpand::tilde(&repo.path);
        let repo_path = Path::new(path.as_ref());

        println!("\n{} ({})", repo.name.yellow().bold(), repo.path);

        if !repo_path.exists() {
            println!("  {} Repository path does not exist", "ERROR:".red().bold());
            continue;
        }

        // Get current branch
        let branch_output = run_git_command(repo_path, &["branch", "--show-current"]);
        let current_branch = String::from_utf8_lossy(&branch_output.stdout)
            .trim()
            .to_string();
        println!("  Current branch: {}", current_branch.cyan());

        // Get status
        let status_output = run_git_command(repo_path, &["status", "--porcelain"]);
        if !status_output.status.success() {
            println!(
                "  {} Failed to get repository status",
                "ERROR:".red().bold()
            );
            continue;
        }

        let status_text = String::from_utf8_lossy(&status_output.stdout);
        if status_text.trim().is_empty() {
            println!("  Status: {}", "Clean".green());
        } else {
            println!("  Status: {}", "Changes detected".yellow());

            // Parse status output for better display
            for line in status_text.lines() {
                if !line.is_empty() {
                    let status_code = &line[0..2];
                    let file_name = &line[3..];

                    let status_desc = match status_code.trim() {
                        "M" => "Modified:".yellow(),
                        "A" => "Added:".green(),
                        "D" => "Deleted:".red(),
                        "R" => "Renamed:".blue(),
                        "C" => "Copied:".cyan(),
                        "U" => "Updated but unmerged:".red(),
                        "??" => "Untracked:".bright_black(),
                        _ => "Changed:".normal(),
                    };

                    println!("    {} {}", status_desc, file_name);
                }
            }
        }

        // Get remote status
        let ahead_behind = run_git_command(
            repo_path,
            &["rev-list", "--count", "--left-right", "@{upstream}...HEAD"],
        );
        if ahead_behind.status.success() {
            let output = String::from_utf8_lossy(&ahead_behind.stdout)
                .trim()
                .to_string();
            if output.contains("\t") {
                let counts: Vec<&str> = output.split('\t').collect();
                if counts.len() == 2 {
                    let behind = counts[0];
                    let ahead = counts[1];

                    if ahead != "0" {
                        println!("  {} {} commit(s) ahead of remote", "↑".green(), ahead);
                    }

                    if behind != "0" {
                        println!("  {} {} commit(s) behind remote", "↓".red(), behind);
                    }

                    if ahead == "0" && behind == "0" {
                        println!("  {} In sync with remote", "=".green());
                    }
                }
            }
        } else {
            // Check if remote exists
            let remote_output = run_git_command(repo_path, &["remote"]);
            if remote_output.status.success()
                && !String::from_utf8_lossy(&remote_output.stdout)
                    .trim()
                    .is_empty()
            {
                println!("  {} No upstream branch set", "!".yellow());
            } else {
                println!("  {} No remote configured", "!".yellow());
            }
        }
    }
}

pub fn sync_repositories(config: &Config, names: &[&str]) {
    let repos = get_repositories_by_names(config, names);

    if repos.is_empty() {
        println!("No repositories to sync.");
        return;
    }

    println!("{}", "Syncing repositories...".green().bold());

    for repo in repos {
        println!("\n{} ({})", repo.name.yellow().bold(), repo.path);

        let path = shellexpand::tilde(&repo.path);
        let repo_path = Path::new(path.as_ref());

        if !repo_path.exists() {
            println!("  {} Repository path does not exist", "ERROR:".red().bold());
            continue;
        }

        // Get current status
        let status_output = run_git_command(repo_path, &["status", "--porcelain"]);
        if !status_output.status.success() {
            println!(
                "  {} Failed to get repository status",
                "ERROR:".red().bold()
            );
            continue;
        }

        let has_changes = !String::from_utf8_lossy(&status_output.stdout)
            .trim()
            .is_empty();

        if has_changes {
            println!("  {} Local changes detected", "WARNING:".yellow().bold());
            // Add all changes
            run_git_command_with_output(repo_path, &["add", "."]);
            // Commit changes
            run_git_command_with_output(
                repo_path,
                &["commit", "-m", "Automatic commit from GitPower"],
            );
        }

        // Pull changes
        let branch = repo.branch.as_deref().unwrap_or("main");
        let remote = repo.remote.as_deref().unwrap_or("origin");

        println!("  Pulling from {}/{}...", remote, branch);

        if has_changes {
            // Push changes
            println!("  Pushing to {}/{}...", remote, branch);
            run_git_command_with_output(repo_path, &["push", remote, branch]);
        }
    }

    println!("\n{}", "Sync complete!".green().bold());
}

pub fn pull_repositories(config: &Config, names: &[&str]) {
    let repos = get_repositories_by_names(config, names);

    if repos.is_empty() {
        println!("No repositories to pull.");
        return;
    }

    println!("{}", "Pulling repositories...".green().bold());

    for repo in repos {
        println!("\n{} ({})", repo.name.yellow().bold(), repo.path);

        let path = shellexpand::tilde(&repo.path);
        let repo_path = Path::new(path.as_ref());

        if !repo_path.exists() {
            println!("  {} Repository path does not exist", "ERROR:".red().bold());
            continue;
        }

        // Pull changes
        let branch = repo.branch.as_deref().unwrap_or("main");
        let remote = repo.remote.as_deref().unwrap_or("origin");

        println!("  Pulling from {}/{}...", remote, branch);

        // Actually perform the pull
        if run_git_command_with_output(repo_path, &["pull", remote, branch]) {
            println!("  {} Pull successful", "SUCCESS:".green().bold());
        } else {
            println!("  {} Pull failed", "ERROR:".red().bold());
        }
    }

    println!("\n{}", "Pull complete!".green().bold());
}

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
        let repo_path = Path::new(path.as_ref());

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
