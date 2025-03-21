use clap::{Arg, Command};

pub fn build_cli() -> Command {
    Command::new("GitPower")
        .version("1.0")
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
}
