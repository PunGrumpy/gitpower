use colored::*;
use crate::config::Config;

pub fn list_repositories(config: &Config) {
    println!("{}", "Configured Repositories:".green().bold());
    for repo in &config.repositories {
        let path = shellexpand::tilde(&repo.path);
        let repo_path = std::path::Path::new(path.as_ref());

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