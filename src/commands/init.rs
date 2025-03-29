use colored::*;
use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use std::fs;
use std::path::Path;

use crate::config::{Config, Group, Repository};
use crate::git::run_git_command_with_output;

pub fn init_repository_interactive(config: &mut Config) {
    let theme = ColorfulTheme::default();

    println!(
        "\n{}",
        "🚀 Welcome to GitPower Repository Initialization!"
            .green()
            .bold()
    );
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
                println!(
                    "{} Repository '{}' already exists in config",
                    "ERROR:".red().bold(),
                    input
                );
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
    init_repository(
        config,
        &name,
        &path,
        remote.as_deref(),
        branch.as_deref(),
        groups.as_deref(),
    );

    println!("\n{}", "✨ Repository setup complete!".green().bold());
}

pub fn init_repository(
    config: &mut Config,
    name: &str,
    path: &str,
    remote: Option<&str>,
    branch: Option<&str>,
    groups: Option<&str>,
) {
    let expanded_path = shellexpand::tilde(path);
    let repo_path = Path::new(expanded_path.as_ref());

    // Check if repository already exists in config
    if config.repositories.iter().any(|r| r.name == name) {
        println!(
            "{} Repository '{}' already exists in config",
            "ERROR:".red().bold(),
            name
        );
        return;
    }

    // Create repository directory if it doesn't exist
    if !repo_path.exists() {
        if let Err(e) = fs::create_dir_all(repo_path) {
            println!(
                "{} Failed to create repository directory: {}",
                "ERROR:".red().bold(),
                e
            );
            return;
        }
        println!("Created repository directory at {}", repo_path.display());
    }

    // Initialize git repository if it doesn't exist
    if !repo_path.join(".git").exists() {
        if !run_git_command_with_output(repo_path, &["init"]) {
            println!(
                "{} Failed to initialize git repository",
                "ERROR:".red().bold()
            );
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
        let group_names: Vec<String> = groups_str
            .split(',')
            .map(|s| s.trim().to_string())
            .collect();
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
    println!(
        "{} Added repository '{}' to config",
        "SUCCESS:".green().bold(),
        name
    );
}
