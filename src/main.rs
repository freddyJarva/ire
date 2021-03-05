/// A simple example demonstrating how to handle user input. This is
/// a bit out of the scope of the library as it does not provide any
/// input handling out of the box. However, it may helps some to get
/// started.
///
/// This is a very simple example:
///   * A input box always focused. Every character you type is registered
///   here
///   * Pressing Backspace erases a character
///   * Pressing Enter pushes the current input in the history of previous
///   messages
// #[allow(dead_code)]
mod util;

use crate::util::event::{Event, Events};
use crate::util::input::{Editable, Input};
use clap::clap_app;
use regex::Regex;
use std::{
    cmp::{max, min},
    error::Error,
    fs, io,
};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};
use unicode_width::UnicodeWidthStr;
use util::input::InputMode;

/// App holds the state of the application
struct App {
    input: Input,
    pattern_matches: Vec<String>,
}

impl Default for App {
    fn default() -> App {
        App {
            input: Input::default(),
            pattern_matches: Vec::new(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(regex_splitter =>
        (version: "1.0")
        (author: "Freddy Järvå <freddy.a.jarva@gmail.com>")
        (about: "Coding Monkey Extraordinaire")
        (@arg FILENAME: +required)
    )
    .get_matches();

    let filename = matches.value_of("FILENAME").unwrap();

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    // Setup event handlers
    let mut events = Events::new();

    // Create default app state
    let mut app = App::default();

    let contents = fs::read_to_string(filename).expect("Unable to read file");

    loop {
        // Draw UI
        terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .margin(2)
                .constraints(
                    [
                        Constraint::Length(1),
                        Constraint::Length(3),
                        Constraint::Min(1),
                    ]
                    .as_ref(),
                )
                .split(f.size());

            let (msg, style) = match app.input.mode {
                InputMode::Normal => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to exit, "),
                        Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to start editing."),
                    ],
                    Style::default().add_modifier(Modifier::RAPID_BLINK),
                ),
                InputMode::Editing => (
                    vec![
                        Span::raw("Press "),
                        Span::styled("Esc", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to stop editing, "),
                        Span::styled("Enter", Style::default().add_modifier(Modifier::BOLD)),
                        Span::raw(" to record the message"),
                    ],
                    Style::default(),
                ),
            };
            let mut text = Text::from(Spans::from(msg));
            text.patch_style(style);
            let help_message = Paragraph::new(text);
            f.render_widget(help_message, chunks[0]);

            let input = Paragraph::new(app.input.text.as_ref())
                .style(match app.input.mode {
                    InputMode::Normal => Style::default(),
                    InputMode::Editing => Style::default().fg(Color::Yellow),
                })
                .block(Block::default().borders(Borders::ALL).title("Input"));
            f.render_widget(input, chunks[1]);
            match app.input.mode {
                InputMode::Normal =>
                    // Hide the cursor. `Frame` does this by default, so we don't need to do anything here
                    {}

                InputMode::Editing => {
                    // Make the cursor visible and ask tui-rs to put it at the specified coordinates after rendering
                    f.set_cursor(
                        // Put cursor past the end of the input text
                        // chunks[1].x + app.input.width() as u16 + 1,
                        chunks[1].x + *app.input.idx() as u16 + 1,
                        // Move one line down, from the border to the input line
                        chunks[1].y + 1,
                    )
                }
            }

            if let Ok(re) = Regex::new(&app.input.text) {
                let pattern_matches: Vec<ListItem> = contents
                    .split('\n')
                    .filter(|s| re.is_match(s))
                    .enumerate()
                    .map(|(i, m)| {
                        let content = vec![Spans::from(Span::raw(format!("{}: {}", i, m)))];
                        ListItem::new(content)
                    })
                    .collect();
                let pattern_matches = List::new(pattern_matches)
                    .block(Block::default().borders(Borders::ALL).title("Messages"));
                f.render_widget(pattern_matches, chunks[2]);
            }
        })?;

        // Handle input
        if let Event::Input(input) = events.next()? {
            match app.input.mode {
                InputMode::Normal => match input {
                    Key::Char('e') => {
                        app.input.mode = InputMode::Editing;
                        events.disable_exit_key();
                    }
                    Key::Char('q') => {
                        break;
                    }
                    _ => {}
                },
                InputMode::Editing => match input {
                    Key::Char('\n') => {
                        app.pattern_matches.push(app.input.text.drain(..).collect());
                    }
                    Key::Char(c) => {
                        app.input.add(c);
                    }
                    Key::Backspace => match app.input.idx() {
                        0 => {}
                        1..=400 => {
                            app.input.delete();
                        }
                        _ => {}
                    },
                    Key::Esc => {
                        app.input.mode = InputMode::Normal;
                        events.enable_exit_key();
                    }
                    Key::Left => {
                        app.input.left();
                    }

                    Key::Right => app.input.right(),
                    _ => {}
                },
            }
        }
    }
    Ok(())
}
