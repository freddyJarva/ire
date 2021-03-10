mod capture;
/// Simple tui that interactively shows matching lines in input
mod color;
mod crate_tests;
mod event;
mod input;

use crate::capture::{filter_matches, into_matchsets, MatchSet};
use crate::color::Styled;
use crate::event::{Event, Events};
use crate::input::{Editable, Input};
use clap::clap_app;
use colored::Colorize;
use csv::Writer;
use glob::glob;
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

use input::InputMode;

/// App holds the state of the application
struct App<'a> {
    input: Input,
    // pattern_matches: Vec<String>,
    pattern_matches: Vec<MatchSet<'a>>,
    re: Regex,
}

impl<'a> Default for App<'a> {
    fn default() -> App<'a> {
        App {
            input: Input::default(),
            // pattern_matches: Vec::new(),
            pattern_matches: Vec::new(),
            re: Regex::new("").unwrap(),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    let matches = clap_app!(interactive_regex =>
        (version: "1.0")
        (author: "Freddy Järvå <freddy.a.jarva@gmail.com>")
        (about: "Coding Monkey Extraordinaire")
        (@arg FILENAME: +required conflicts_with[GLOB])
        (@arg GLOB: -g --glob +takes_value "use glob pattern to read from multiple files")
        (@arg OUTPUT: -o --output +takes_value "write result to file")
    )
    .get_matches();

    let contents: Vec<String> = if let Some(glob_pattern) = matches.value_of("GLOB") {
        let mut strings: Vec<String> = Vec::new();
        for entry in glob(glob_pattern).unwrap() {
            let file_content = fs::read_to_string(entry.unwrap()).unwrap();
            strings.extend(file_content.split('\n').map(|s| s.to_string()));
        }
        strings
    } else {
        let filename = matches.value_of("FILENAME").unwrap();
        fs::read_to_string(filename)
            .expect(&format!("Unable to read file \"{}\"", filename))
            .split("\n")
            .map(|s| s.to_string())
            .collect()
    };

    // Terminal initialization
    let stdout = io::stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let terminal = Terminal::new(backend)?;

    let events = Events::new();

    let app = App::default();

    match begin_loop(terminal, app, contents, events) {
        // matches execute when exiting the program
        Ok((contents, re)) => {
            let mats = filter_matches(&contents, &re);
            let mats = into_matchsets(&mats, &re);
            if let Some(output) = matches.value_of("OUTPUT") {
                let mut writer = Writer::from_path(output).unwrap();
                for line in mats {
                    writer.write_record(line.to_strings())?;
                }
            } else {
                let stdout = io::stdout();
                let mut handle = io::BufWriter::new(stdout.lock());
                for line in mats {
                    writeln!(handle, "{}", line.raw_line())?;
                }
                writeln!(handle, "Lines were matched with: {}", re.as_str().green())?;
            }
        }
        Err(err) => {
            eprintln!("program crash: {}", err)
        }
    }

    Ok(())
}

fn begin_loop<'a>(
    mut terminal: Terminal<
        TermionBackend<AlternateScreen<MouseTerminal<termion::raw::RawTerminal<io::Stdout>>>>,
    >,
    mut app: App<'a>,
    contents: Vec<String>,
    mut events: Events,
) -> Result<(Vec<String>, Regex), Box<dyn Error>> {
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
                    Ok(re) => app.re = re,
                    Err(_) => {}
                }
                let matches = filter_matches(&contents, &app.re);
                let pattern_matches = into_matchsets(&matches, &app.re);
                let pattern_matches: Vec<ListItem> = pattern_matches
                    .iter()
                    .map(|color_styles| color_styles.style())
                    .map(|spans| ListItem::new(Spans::from(spans)))
                    .collect();
                let pattern_matches = List::new(pattern_matches)
                    .block(Block::default().borders(Borders::ALL).title("Messages"));
                f.render_widget(pattern_matches, chunks[2]);
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
                    Key::Char('\n') => return Ok((contents.to_vec(), app.re)),
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
