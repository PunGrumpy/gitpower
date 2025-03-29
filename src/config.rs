use colored::*;
use serde::{Deserialize, Serialize};
use std::error::Error;
use std::fs;
use std::path::Path;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub repositories: Vec<Repository>,
    pub groups: Option<Vec<Group>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Repository {
    pub name: String,
    pub path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<String>>,
}

impl Repository {
    pub fn validate(&self) -> Result<(), String> {
        if let Some(remote) = &self.remote {
            if !remote.starts_with("http://") && !remote.starts_with("https://") && !remote.starts_with("git@") {
                return Err(format!("Invalid remote URL format: {}", remote));
            }
        }
        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Group {
    pub name: String,
    pub repositories: Vec<String>,
}

pub fn load_config(config_path: &str) -> Result<Config, Box<dyn Error>> {
    match fs::read_to_string(config_path) {
        Ok(content) => match serde_yaml::from_str::<Config>(&content) {
            Ok(config) => Ok(config),
            Err(e) => {
                eprintln!("{}: {}", "Error parsing config".red(), e);
                create_default_config(config_path);
                Err(Box::new(e))
            }
        },
        Err(e) => {
            eprintln!("{}: {}", "Config file not found, creating default config".yellow(), e);
            create_default_config(config_path);
            // Try to load the newly created config
            match fs::read_to_string(config_path) {
                Ok(content) => match serde_yaml::from_str::<Config>(&content) {
                    Ok(config) => Ok(config),
                    Err(e) => Err(Box::new(e))
                },
                Err(e) => Err(Box::new(e))
            }
        }
    }
}

pub fn create_default_config(config_path: &str) {
    let path = Path::new(config_path);

    // Create directory if it doesn't exist
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            if let Err(e) = fs::create_dir_all(parent) {
                eprintln!("Failed to create config directory: {}", e);
                return;
            }
        }
    }

    let default_config = Config {
        repositories: vec![Repository {
            name: "example-repo".to_string(),
            path: "~/repos/example".to_string(),
            remote: Some("origin".to_string()),
            branch: Some("main".to_string()),
            groups: Some(vec!["default".to_string()]),
        }],
        groups: Some(vec![Group {
            name: "default".to_string(),
            repositories: vec!["example-repo".to_string()],
        }]),
    };

    match serde_yaml::to_string(&default_config) {
        Ok(yaml) => {
            if let Err(e) = fs::write(path, yaml) {
                eprintln!("Failed to write default config: {}", e);
            } else {
                println!("Created default config at {}", config_path);
                println!("Please edit this file to add your repositories.");
            }
        }
        Err(e) => eprintln!("Failed to serialize default config: {}", e),
    }
}

pub fn get_repositories_by_names<'a>(config: &'a Config, names: &[&str]) -> Vec<&'a Repository> {
    if names.is_empty() {
        return config.repositories.iter().collect();
    }

    let mut result = Vec::new();
    let empty_groups = Vec::new();
    let groups = config.groups.as_ref().unwrap_or(&empty_groups);

    for name in names {
        // Check if name is a group
        let group_repos: Vec<&str> = groups
            .iter()
            .filter(|g| g.name == *name)
            .flat_map(|g| g.repositories.iter().map(|r| r.as_str()))
            .collect();

        if !group_repos.is_empty() {
            // It's a group, add all repositories in this group
            for repo_name in group_repos {
                if let Some(repo) = config.repositories.iter().find(|r| r.name == repo_name) {
                    if !result.contains(&repo) {
                        result.push(repo);
                    }
                } else {
                    eprintln!("{}: Repository '{}' in group '{}' not found", 
                        "Warning".yellow(), repo_name, name);
                }
            }
        } else {
            // It's a repository name
            if let Some(repo) = config.repositories.iter().find(|r| r.name == *name) {
                if !result.contains(&repo) {
                    result.push(repo);
                }
            } else {
                eprintln!("{}: Repository or group '{}' not found", 
                    "Warning".yellow(), name);
            }
        }
    }

    if result.is_empty() {
        eprintln!("{}: No valid repositories found for the specified names", 
            "Warning".yellow());
    }

    result
}
