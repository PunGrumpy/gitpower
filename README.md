# 🚀 GitPower

A powerful CLI tool for managing multiple Git repositories simultaneously, developed in Rust.

![Preview](./public/preview.png)

## ✨ Features

- **Sync multiple repositories simultaneously** (pull and push) 🔄
- **Pull changes from all repositories** with a single command 📥
- **Run commands** across multiple repositories 💻
- **Group repositories** for easier management 📁
- **Configure via YAML** for simple setup ⚙️
- **Initialize new repositories** with a single command 🆕
- **Completion for shell** (bash, zsh, fish) 🐚
- **Interactive mode** for easier navigation 🕹️

## 📦 Installation

### From Source Code

1. Clone the project:

   ```
   git clone https://github.com/PunGrumy/gitpower.git
   cd gitpower
   ```

2. Build with Cargo:

   ```
   cargo build --release
   ```

3. Move the binary to your PATH:

   ```
   sudo mv target/release/gitpower /usr/local/bin/
   ```

### Via Cargo

```
cargo install gitpower
```

## 🛠️ Configuration

GitPower uses a YAML file to configure repositories and groups. By default, it looks for a configuration file at `~/.config/gitpower/config.yml`

Basic configuration example:

```yaml
repositories:
  - name: project-a
    path: ~/projects/project-a
    remote: origin
    branch: main
    groups:
      - work

  - name: dotfiles
    path: ~/dotfiles
    remote: origin
    branch: master
    groups:
      - personal

groups:
  - name: work
    repositories:
      - project-a

  - name: personal
    repositories:
      - dotfiles
```

## 📚 Usage

### Basic Commands

| Command                | Description                                             |
| ---------------------- | ------------------------------------------------------- |
| `gitpower init`        | Start interactive dialog to initialize a new repository |
| `gitpower list`        | List configured repositories                            |
| `gitpower sync`        | Sync all repositories                                   |
| `gitpower pull`        | Pull changes from all repositories                      |
| `gitpower interactive` | Start interactive mode                                  |
| `gitpower help`        | Show help message                                       |

### Sync Commands

| Command                            | Description                |
| ---------------------------------- | -------------------------- |
| `gitpower sync project-a dotfiles` | Sync specific repositories |
| `gitpower sync work personal`      | Sync repositories by group |

### Run Commands

| Command                                   | Description                                    |
| ----------------------------------------- | ---------------------------------------------- |
| `gitpower run "git status"`               | Run command in all repositories                |
| `gitpower run "npm install" web-projects` | Run command in specific repositories or groups |

### Configuration

| Command                                         | Description                   |
| ----------------------------------------------- | ----------------------------- |
| `gitpower --config ~/my-custom-config.yml list` | Use custom configuration file |

### Interactive Mode

In interactive mode, you can navigate through repositories and groups with arrow keys and execute commands with a single key press.

## 🔍 Advanced Usage

### Automatic Syncing with Cron

You can set up a cron job for automatic repository syncing:

```
# Sync every hour
0 * * * * gitpower sync
```

### Shell Aliases

For even quicker access, you can set up shell aliases:

```bash
# In your .bashrc or .zshrc
alias gp="gitpower sync"
alias gpp="gitpower pull"
alias gpr="gitpower run"
```

## 👥 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.
