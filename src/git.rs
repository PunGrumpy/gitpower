use colored::*;
use std::os::unix::process::ExitStatusExt;
use std::path::Path;
use std::process::{Command, Output};

pub fn run_git_command(repo_path: &Path, args: &[&str]) -> Output {
    Command::new("git")
        .current_dir(repo_path)
        .args(args)
        .output()
        .unwrap_or_else(|_| Output {
            status: std::process::ExitStatus::from_raw(1),
            stdout: Vec::new(),
            stderr: Vec::new(),
        })
}

pub fn run_git_command_with_output(repo_path: &Path, args: &[&str]) -> bool {
    let output = run_git_command(repo_path, args);

    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    if !stdout.trim().is_empty() {
        // Better formatting for command output
        if stdout.lines().count() > 1 {
            println!("  Output:");
            for line in stdout.lines() {
                println!("    {}", line);
            }
        } else {
            println!("  Output: {}", stdout.trim());
        }
    }

    if !stderr.trim().is_empty() {
        // Better formatting for error output
        if stderr.lines().count() > 1 {
            eprintln!("  Errors:");
            for line in stderr.lines() {
                eprintln!("    {}", line.red());
            }
        } else {
            eprintln!("  Error: {}", stderr.trim().red());
        }
    }

    output.status.success()
}
