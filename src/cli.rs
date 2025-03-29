use clap::{Arg, Command};
use clap_complete::{Shell, generate};

pub fn build_cli() -> Command {
    Command::new("GitPower")
        .version(env!("CARGO_PKG_VERSION"))
        .author("PunGrumpy")
        .about("Manage multiple Git repositories effortlessly")
        .arg(
            Arg::new("config")
                .short('c')
                .long("config")
                .value_name("FILE")
                .help("Sets a custom config file"),
        )
        .subcommand(Command::new("list").about("List all configured repositories"))
        .subcommand(
            Command::new("init")
                .about("Initialize a new repository or add existing repository to config")
                .arg(
                    Arg::new("name")
                        .help("Name of the repository")
                        .required(false),
                )
                .arg(
                    Arg::new("path")
                        .help("Path to the repository")
                        .required(false),
                )
                .arg(
                    Arg::new("remote")
                        .help("Remote URL (optional)")
                        .long("remote"),
                )
                .arg(
                    Arg::new("branch")
                        .help("Default branch (optional)")
                        .long("branch"),
                )
                .arg(
                    Arg::new("groups")
                        .help("Groups to add the repository to (comma-separated)")
                        .long("groups"),
                )
                .arg(
                    Arg::new("non-interactive")
                        .help("Run in non-interactive mode")
                        .long("non-interactive")
                        .short('n')
                        .action(clap::ArgAction::SetTrue),
                ),
        )
        .subcommand(
            Command::new("status")
                .about("Show status of all repositories")
                .arg(
                    Arg::new("repos")
                        .help("Specific repositories or groups to check status")
                        .action(clap::ArgAction::Append),
                ),
        )
        .subcommand(
            Command::new("sync")
                .about("Sync repositories (pull, push)")
                .arg(
                    Arg::new("repos")
                        .help("Specific repositories or groups to sync")
                        .action(clap::ArgAction::Append),
                ),
        )
        .subcommand(
            Command::new("pull").about("Pull from repositories").arg(
                Arg::new("repos")
                    .help("Specific repositories or groups to pull")
                    .action(clap::ArgAction::Append),
            ),
        )
        .subcommand(
            Command::new("run")
                .about("Run a command in all repositories")
                .arg(Arg::new("command").help("Command to run").required(true))
                .arg(
                    Arg::new("repos")
                        .help("Specific repositories or groups to run in")
                        .action(clap::ArgAction::Append),
                ),
        )
        .subcommand(
            Command::new("interactive")
                .about("Launch interactive mode (like LazyGit)")
                .alias("i"),
        )
        .subcommand(
            Command::new("completion")
                .about("Generate shell completion")
                .arg(
                    Arg::new("shell")
                        .help("Shell to generate completion for")
                        .value_parser(["bash", "zsh", "fish", "powershell"])
                        .required(true),
                ),
        )
}

pub fn print_completion(shell: Shell) {
    let mut cmd = build_cli();
    let name = cmd.get_name().to_string();
    generate(shell, &mut cmd, name, &mut std::io::stdout());
}
