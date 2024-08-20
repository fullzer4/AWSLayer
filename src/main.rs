use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
    text::Span,
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Terminal,
};
use crossterm::{
    event::{self, Event, KeyCode},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::{env, fs, io};

struct StatefulList {
    state: ListState,
    items: Vec<String>,
}

impl StatefulList {
    fn with_items(items: Vec<String>) -> StatefulList {
        StatefulList {
            state: ListState::default(),
            items,
        }
    }

    fn next(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i >= self.items.len() - 1 {
                    0
                } else {
                    i + 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }

    fn previous(&mut self) {
        let i = match self.state.selected() {
            Some(i) => {
                if i == 0 {
                    self.items.len() - 1
                } else {
                    i - 1
                }
            }
            None => 0,
        };
        self.state.select(Some(i));
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut language_list = StatefulList::with_items(vec!["Python".into()]);
    let mut arch_list = StatefulList::with_items(vec!["arm64".into(), "x86_64".into()]);
    let mut zip_name = String::new();

    loop {
        terminal.draw(|f| {
            // Layout das 3 colunas
            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(
                    [
                        Constraint::Percentage(20), // Coluna esquerda
                        Constraint::Percentage(50), // Coluna central
                        Constraint::Percentage(30), // Coluna direita
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let languages: Vec<ListItem> = language_list.items.iter().map(|i| ListItem::new(i.as_str())).collect();
            let languages = List::new(languages)
                .block(Block::default().borders(Borders::ALL).title("Linguagem"))
                .highlight_style(Style::default().bg(Color::LightGreen))
                .highlight_symbol(">> ");

            f.render_stateful_widget(languages, chunks[0], &mut language_list.state);

            let files = get_txt_files().unwrap_or_default();
            let files: Vec<ListItem> = files.iter().map(|f| ListItem::new(f.as_str())).collect();
            let files_list = List::new(files)
                .block(Block::default().borders(Borders::ALL).title("Arquivos .txt"))
                .highlight_style(Style::default().bg(Color::LightGreen))
                .highlight_symbol(">> ");

            f.render_widget(files_list, chunks[1]);

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)].as_ref())
                .split(chunks[2]);

            let architectures: Vec<ListItem> = arch_list.items.iter().map(|i| ListItem::new(i.as_str())).collect();
            let arch_widget = List::new(architectures)
                .block(Block::default().borders(Borders::ALL).title("Arquitetura"))
                .highlight_style(Style::default().bg(Color::LightGreen))
                .highlight_symbol(">> ");
            f.render_stateful_widget(arch_widget, right_chunks[0], &mut arch_list.state);

            let zip_paragraph = Paragraph::new(Span::from(zip_name.as_str()))
                .block(Block::default().borders(Borders::ALL).title("Nome do ZIP"));
            f.render_widget(zip_paragraph, right_chunks[1]);
        })?;

        if let Event::Key(key) = event::read()? {
            match key.code {
                KeyCode::Char(c) => {
                    zip_name.push(c);
                }
                KeyCode::Backspace => {
                    zip_name.pop();
                }
                KeyCode::Enter => {
                    break;
                }
                KeyCode::Up => {
                    arch_list.previous();
                }
                KeyCode::Down => {
                    arch_list.next();
                }
                KeyCode::Esc => {
                    // Sai do programa
                    disable_raw_mode()?;
                    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
                    terminal.show_cursor()?;
                    return Ok(());
                }
                _ => {}
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen)?;
    terminal.show_cursor()?;

    Ok(())
}

fn get_txt_files() -> io::Result<Vec<String>> {
    let mut txt_files = Vec::new();
    let current_dir = env::current_dir()?;
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() {
            if let Some(extension) = path.extension() {
                if extension == "txt" {
                    if let Some(filename) = path.file_name() {
                        txt_files.push(filename.to_string_lossy().into_owned());
                    }
                }
            }
        }
    }
    Ok(txt_files)
}