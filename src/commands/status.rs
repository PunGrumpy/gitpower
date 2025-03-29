use crate::config::{Config, get_repositories_by_names};
use crate::git::run_git_command;
use colored::*;

pub fn show_repository_status(config: &Config, names: &[&str]) {
    let repos = get_repositories_by_names(config, names);

    if repos.is_empty() {
        println!("No repositories to check status.");
        return;
    }

    println!("{}", "Repository Status:".green().bold());

    for repo in repos {
        let path = shellexpand::tilde(&repo.path);
        let repo_path = std::path::Path::new(path.as_ref());

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
