use nu_ansi_term::{Color, Style};
use reedline::{Highlighter, StyledText};

pub struct MusicmanHighlighter {
    pub commands: Vec<&'static str>,
    pub subcommands: Vec<(&'static str, Vec<&'static str>)>,
    pub takes_parameters: Vec<&'static str>,
}

impl MusicmanHighlighter {
    pub fn new() -> Self {
        Self {
            commands: vec![
                "clear", "exit", "ls", "next", "p", "pause", "pl", "playlist", "prev", "replay",
                "search", "show",
            ],
            subcommands: vec![
                ("pl", vec!["new", "load", "show", "ls"]),
                ("playlist", vec!["new", "load", "show", "ls"]),
                ("search", vec!["artist", "a", "title", "t"]),
            ],
            takes_parameters: vec![
                "pl load",
                "pl new",
                "playlist load",
                "playlist new",
                "search a",
                "search t",
                "search artist",
                "search title",
            ],
        }
    }
}

impl Highlighter for MusicmanHighlighter {
    fn highlight(&self, line: &str, _cursor: usize) -> StyledText {
        let mut out = StyledText::new();

        for (i, chunk) in line.split_inclusive(|c| c == ' ').enumerate() {
            let trimmed = chunk.trim_end();

            let style = if i == 0 {
                if self.commands.contains(&trimmed) {
                    Style::new().fg(Color::Blue).bold()
                } else {
                    Style::new().fg(Color::Red)
                }
            } else if i == 1 {
                let prev = line.split_whitespace().next().unwrap_or("");

                let sub_known = self
                    .subcommands
                    .iter()
                    .find(|(c, _)| *c == prev)
                    .map(|(_, subs)| subs.contains(&trimmed))
                    .unwrap_or(false);

                if sub_known {
                    Style::new().fg(Color::LightPurple).bold()
                } else {
                    Style::new().fg(Color::Red)
                }
            } else {
                let cmd = line
                    .split_whitespace()
                    .take(2)
                    .collect::<Vec<&str>>()
                    .join(" ");

                let is_param = self.takes_parameters.contains(&cmd.trim());
                if is_param {
                    Style::new().fg(Color::White).bold()
                } else {
                    Style::new().fg(Color::DarkGray).italic()
                }
            };
            out.push((style, chunk.to_string()));
        }
        out
    }
}
