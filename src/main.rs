/// Simple tui that interactively shows matching lines in input
mod util;

use crate::util::color::{collect_matches, filter_matches, Styled};
use crate::util::event::{Event, Events};
use crate::util::input::{Editable, Input};
use clap::clap_app;
use colored::Colorize;
use regex::Regex;
use std::io::Write;
use std::{error::Error, fs, io};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Terminal,
};

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
        (@arg TEST: -t --test +takes_value "test stuff")
    )
    .get_matches();

    let test = matches.value_of("TEST");
    match test {
        Some(s) => {
            println!("Hello {}!", s.red());
            return Ok(());
        }
        None => {}
    }

    let filename = matches.value_of("FILENAME").unwrap();
    let contents: Vec<String> = fs::read_to_string(filename)
        .expect(&format!("Unable to read file \"{}\"", filename))
        .split("\n")
        .map(|s| s.to_string())
        .collect();

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    // Setup event handlers
    let events = Events::new();

    // Create default app state
    let app = App::default();

    match begin_loop(terminal, app, contents, events) {
        Ok(mats) => {
            let stdout = io::stdout();
            let mut handle = io::BufWriter::new(stdout.lock());
            for line in mats {
                writeln!(handle, "{}", line)?;
            }
        }
        Err(err) => {
            eprintln!("program crash: {}", err)
        }
    }

    Ok(())
}

fn begin_loop(
    mut terminal: Terminal<
        TermionBackend<AlternateScreen<MouseTerminal<termion::raw::RawTerminal<io::Stdout>>>>,
    >,
    mut app: App,
    contents: Vec<String>,
    mut events: Events,
) -> Result<Vec<String>, Box<dyn Error>> {
    loop {
        // Draw UI
        terminal
            .draw(|f| {
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
                            Span::styled("i", Style::default().add_modifier(Modifier::BOLD)),
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
                            chunks[1].x + *app.input.idx() as u16 + 1,
                            // Move one line down, from the border to the input line
                            chunks[1].y + 1,
                        )
                    }
                }

                match Regex::new(&app.input.text) {
                    Ok(re) => {
                        app.pattern_matches = filter_matches(&contents, &re);
                        let pattern_matches = collect_matches(&contents, &re);
                        let pattern_matches: Vec<ListItem> = pattern_matches
                            .iter()
                            .map(|color_styles| color_styles.style())
                            .map(|spans| ListItem::new(spans))
                            .collect();
                        let pattern_matches = List::new(pattern_matches)
                            .block(Block::default().borders(Borders::ALL).title("Messages"));
                        f.render_widget(pattern_matches, chunks[2]);
                    }
                    Err(_) => {
                        // Don't update match screen until proper match is found
                    }
                }
            })
            .expect("Failure on draw");

        // Handle input
        if let Event::Input(input) = events.next().expect("Failure on input") {
            match app.input.mode {
                InputMode::Normal => match input {
                    Key::Char('i') => {
                        app.input.mode = InputMode::Editing;
                        events.disable_exit_key();
                    }
                    Key::Char('q') => {
                        panic!("Exiting without writing result")
                    }
                    _ => {}
                },
                InputMode::Editing => match input {
                    Key::Char('\n') => return Ok(app.pattern_matches),
                    Key::Alt(',') => app.input.previous_boundary(),
                    Key::Alt('.') => app.input.next_boundary(),
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
                    Key::Home => app.input.home(),
                    Key::End => app.input.end(),
                    _ => {}
                },
            }
        }
    }
}
