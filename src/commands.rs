use colored::*;
use std::path::Path;
use std::process::Command as ProcessCommand;
use std::fs;
use dialoguer::{theme::ColorfulTheme, Input, Confirm};

use crate::config::{Config, Repository, Group};
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

pub fn init_repository_interactive(config: &mut Config) {
    let theme = ColorfulTheme::default();

    println!("\n{}", "🚀 Welcome to GitPower Repository Initialization!".green().bold());
    println!("This will help you set up a new repository or add an existing one.\n");

    // Repository Name
    println!("{}", "Repository Name".cyan().bold());
    let name = loop {
        let input: String = Input::with_theme(&theme)
            .with_prompt("What is the name of your repository?")
            .interact_text()
            .unwrap();

        if !input.is_empty() {
            if config.repositories.iter().any(|r| r.name == input) {
                println!("{} Repository '{}' already exists in config", "ERROR:".red().bold(), input);
                continue;
            }
            break input;
        }
        println!("Repository name cannot be empty");
    };

    // Repository Path
    println!("\n{}", "Repository Path".cyan().bold());
    let path = loop {
        let input: String = Input::with_theme(&theme)
            .with_prompt("Where would you like to create the repository?")
            .interact_text()
            .unwrap();

        if !input.is_empty() {
            break input;
        }
        println!("Repository path cannot be empty");
    };

    // Remote Repository
    println!("\n{}", "Remote Repository".cyan().bold());
    let remote = if Confirm::with_theme(&theme)
        .with_prompt("Would you like to add a remote repository?")
        .interact()
        .unwrap()
    {
        Some(
            Input::<String>::with_theme(&theme)
                .with_prompt("Enter the remote URL")
                .interact_text()
                .unwrap(),
        )
    } else {
        None
    };

    // Default Branch
    println!("\n{}", "Default Branch".cyan().bold());
    let branch = if Confirm::with_theme(&theme)
        .with_prompt("Would you like to set a default branch?")
        .interact()
        .unwrap()
    {
        Some(
            Input::<String>::with_theme(&theme)
                .with_prompt("Enter the default branch name (e.g., main)")
                .interact_text()
                .unwrap(),
        )
    } else {
        None
    };

    // Repository Groups
    println!("\n{}", "Repository Groups".cyan().bold());
    let groups = if Confirm::with_theme(&theme)
        .with_prompt("Would you like to add this repository to any groups?")
        .interact()
        .unwrap()
    {
        Some(
            Input::<String>::with_theme(&theme)
                .with_prompt("Enter group names (comma-separated)")
                .interact_text()
                .unwrap(),
        )
    } else {
        None
    };

    // Initialize the repository
    println!("\n{}", "Initializing repository...".cyan().bold());
    init_repository(config, &name, &path, remote.as_deref(), branch.as_deref(), groups.as_deref());

    println!("\n{}", "✨ Repository setup complete!".green().bold());
}

pub fn init_repository(config: &mut Config, name: &str, path: &str, remote: Option<&str>, branch: Option<&str>, groups: Option<&str>) {
    let expanded_path = shellexpand::tilde(path);
    let repo_path = Path::new(expanded_path.as_ref());

    // Check if repository already exists in config
    if config.repositories.iter().any(|r| r.name == name) {
        println!("{} Repository '{}' already exists in config", "ERROR:".red().bold(), name);
        return;
    }

    // Create repository directory if it doesn't exist
    if !repo_path.exists() {
        if let Err(e) = fs::create_dir_all(repo_path) {
            println!("{} Failed to create repository directory: {}", "ERROR:".red().bold(), e);
            return;
        }
        println!("Created repository directory at {}", repo_path.display());
    }

    // Initialize git repository if it doesn't exist
    if !repo_path.join(".git").exists() {
        if !run_git_command_with_output(repo_path, &["init"]) {
            println!("{} Failed to initialize git repository", "ERROR:".red().bold());
            return;
        }
        println!("Initialized git repository");
    }

    // Add remote if provided
    if let Some(remote_url) = remote {
        if !run_git_command_with_output(repo_path, &["remote", "add", "origin", remote_url]) {
            println!("{} Failed to add remote", "ERROR:".red().bold());
            return;
        }
        println!("Added remote: {}", remote_url);
    }

    // Create new repository entry
    let mut new_repo = Repository {
        name: name.to_string(),
        path: path.to_string(),
        remote: remote.map(|s| s.to_string()),
        branch: branch.map(|s| s.to_string()),
        groups: None,
    };

    // Handle groups
    if let Some(groups_str) = groups {
        let group_names: Vec<String> = groups_str.split(',').map(|s| s.trim().to_string()).collect();
        new_repo.groups = Some(group_names.clone());

        // Create or update groups
        for group_name in group_names {
            if let Some(groups) = &mut config.groups {
                if let Some(group) = groups.iter_mut().find(|g| g.name == group_name) {
                    if !group.repositories.contains(&name.to_string()) {
                        group.repositories.push(name.to_string());
                    }
                } else {
                    groups.push(Group {
                        name: group_name.clone(),
                        repositories: vec![name.to_string()],
                    });
                }
            } else {
                config.groups = Some(vec![Group {
                    name: group_name.clone(),
                    repositories: vec![name.to_string()],
                }]);
            }
        }
    }

    // Add repository to config
    config.repositories.push(new_repo);
    println!("{} Added repository '{}' to config", "SUCCESS:".green().bold(), name);
}
