use colored::*;
use crate::config::{Config, get_repositories_by_names};
use crate::git::{run_git_command, run_git_command_with_output};

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
        let repo_path = std::path::Path::new(path.as_ref());

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
        let repo_path = std::path::Path::new(path.as_ref());

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