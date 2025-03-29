use anyhow::Result;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, ListState},
    Terminal,
};
use std::io;
use std::path::Path;

use crate::config::Config;
use crate::git::run_git_command;

pub struct App {
    repositories: Vec<String>,
    selected_index: usize,
    status_text: String,
    list_state: ListState,
    config: Config,
}

impl App {
    pub fn new(config: Config) -> Self {
        let repositories = config
            .repositories
            .iter()
            .map(|repo| repo.name.clone())
            .collect();
        Self {
            repositories,
            selected_index: 0,
            status_text: String::new(),
            list_state: ListState::default().with_selected(Some(0)),
            config,
        }
    }

    pub fn run(&mut self) -> Result<()> {
        // Terminal initialization
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let mut terminal = Terminal::new(backend)?;

        // Create app and run it
        let res = self.run_app(&mut terminal);

        // Restore terminal
        disable_raw_mode()?;
        execute!(
            terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        terminal.show_cursor()?;

        if let Err(err) = res {
            println!("{err:?}");
        }

        Ok(())
    }

    fn run_app(&mut self, terminal: &mut Terminal<CrosstermBackend<io::Stdout>>) -> Result<()> {
        let mut should_quit = false;
        while !should_quit {
            terminal.draw(|f| self.ui(f))?;
            should_quit = self.handle_events()?;
        }
        Ok(())
    }

    fn ui(&mut self, f: &mut ratatui::Frame) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                [
                    Constraint::Length(3),  // Title
                    Constraint::Min(10),    // Repository List
                    Constraint::Length(6),  // Status Text
                    Constraint::Length(3),  // Status Bar
                ]
                .as_ref(),
            )
            .split(f.area());

        // Title
        let title = Paragraph::new(Line::from(vec![
            Span::styled(
                "GitPower Interactive Mode",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        ]))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(title, chunks[0]);

        // Repository List
        let items: Vec<ListItem> = self
            .repositories
            .iter()
            .enumerate()
            .map(|(i, repo)| {
                let style = if i == self.selected_index {
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::REVERSED)
                } else {
                    Style::default().fg(Color::White)
                };
                
                ListItem::new(Line::from(vec![
                    Span::styled(
                        format!(" {} ", repo),
                        style,
                    ),
                ]))
            })
            .collect();

        let repos = List::new(items)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Repositories"));
        f.render_stateful_widget(repos, chunks[1], &mut self.list_state);

        // Status Text (keep the colored status text)
        let status_lines: Vec<Line> = self.status_text
            .split('\n')
            .map(|line| {
                if line.starts_with("Repository:") {
                    Line::from(vec![
                        Span::styled("Repository: ", Style::default().fg(Color::Green)),
                        Span::styled(line.strip_prefix("Repository: ").unwrap_or(line), Style::default().fg(Color::White)),
                    ])
                } else if line.starts_with("Branch:") {
                    Line::from(vec![
                        Span::styled("Branch: ", Style::default().fg(Color::Yellow)),
                        Span::styled(line.strip_prefix("Branch: ").unwrap_or(line), Style::default().fg(Color::White)),
                    ])
                } else if line.starts_with("Remote:") {
                    Line::from(vec![
                        Span::styled("Remote: ", Style::default().fg(Color::Blue)),
                        Span::styled(line.strip_prefix("Remote: ").unwrap_or(line), Style::default().fg(Color::White)),
                    ])
                } else {
                    Line::from(vec![Span::styled(line, Style::default().fg(Color::White))])
                }
            })
            .collect();

        let status_text = Paragraph::new(status_lines)
            .block(Block::default()
                .borders(Borders::ALL)
                .title("Status"));
        f.render_widget(status_text, chunks[2]);

        // Status Bar
        let status = Paragraph::new(Line::from(vec![
            Span::styled(
                "↑↓: Navigate | Enter: Select | q: Quit",
                Style::default()
                    .fg(Color::White)
                    .bg(Color::Blue),
            ),
        ]))
        .alignment(ratatui::layout::Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
        f.render_widget(status, chunks[3]);
    }

    fn handle_events(&mut self) -> Result<bool> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') => return Ok(true),
                    KeyCode::Up => {
                        if self.selected_index > 0 {
                            self.selected_index -= 1;
                            self.list_state.select(Some(self.selected_index));
                        }
                    }
                    KeyCode::Down => {
                        if self.selected_index < self.repositories.len() - 1 {
                            self.selected_index += 1;
                            self.list_state.select(Some(self.selected_index));
                        }
                    }
                    KeyCode::Enter => {
                        self.show_repository_status(self.selected_index);
                    }
                    _ => {}
                }
            }
        }
        Ok(false)
    }

    fn show_repository_status(&mut self, index: usize) {
        let repo_name = &self.repositories[index];
        let repo = self.config.repositories.iter().find(|r| r.name == *repo_name).unwrap();
        let path = shellexpand::tilde(&repo.path);
        let repo_path = Path::new(path.as_ref());

        if !repo_path.exists() {
            self.status_text = format!("Error: Repository path does not exist: {}", repo.path);
            return;
        }

        // Get current branch
        let branch_output = run_git_command(repo_path, &["branch", "--show-current"]);
        let current_branch = String::from_utf8_lossy(&branch_output.stdout).trim().to_string();

        // Get status
        let status_output = run_git_command(repo_path, &["status", "--porcelain"]);
        let status_text = String::from_utf8_lossy(&status_output.stdout);
        let has_changes = !status_text.trim().is_empty();

        // Get remote info (only first line)
        let remote_output = run_git_command(repo_path, &["remote", "-v"]);
        let remote_info = String::from_utf8_lossy(&remote_output.stdout);
        let remote_url = remote_info.lines().next().unwrap_or("No remote");

        self.status_text = format!(
            "Repository: {}\nBranch: {} | Changes: {}\nRemote: {}",
            repo_name,
            current_branch,
            if has_changes { "Yes" } else { "No" },
            remote_url
        );
    }
} 