use crate::{handlers::*, types::*};
use anyhow::Result;
use colored::Colorize;
use reedline::{Reedline, Signal};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};

pub fn musicman_prompt(
    stream: &TcpStream,
    state: &Arc<Mutex<ClientStateStruct>>,
    sink: &RodioSink,
    editor: &mut Reedline,
) -> Result<()> {
    let prompt = MusicmanPrompt {
        left: "musicman❯ ".green().bold().to_string(),
    };

    let sig = editor.read_line(&prompt);

    let input = match sig {
        Ok(Signal::Success(buffer)) => buffer.trim().to_string(),
        Ok(Signal::CtrlC) => {
            println!("{}", "Use CtrlD or type exit to quit.".yellow().bold());
            return Ok(());
        }
        Ok(Signal::CtrlD) => {
            println!("{}", "Exiting...".green());
            sink.lock().unwrap().stop();
            return Err(anyhow::anyhow!("Exit"));
        }
        Err(e) => return Err(e.into()),
    };

    let input = input
        .split_ascii_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    if input.is_empty() {
        return Ok(());
    }

    match input[0].as_str() {
        "replay" => handle_replay(stream, state),
        "pause" | "p" => handle_pause(sink),
        "clear" => handle_clear(state, sink),
        "next" | "prev" => handle_next_prev(stream, state, input),
        "show" | "ls" => handle_show(state),
        "playlist" | "pl" => handle_playlist(stream, input, state),
        "search" => handle_search(stream, input),
        "exit" => {
            println!("{}", "Exiting...".green());
            sink.lock().unwrap().stop();
            return Err(anyhow::anyhow!("Exit"));
        }
        cmd => {
            println!("{} {}", "Unknown command:".red(), cmd.red().bold());
            print_help();
        }
    }

    Ok(())
}
