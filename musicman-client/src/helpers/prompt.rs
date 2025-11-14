use crate::{handlers::*, types::*};
use anyhow::Result;
use colored::Colorize;
use std::{
    io::{Write, stdin, stdout},
    net::TcpStream,
    sync::{Arc, Mutex},
};

pub fn handle_prompt(
    stream: &TcpStream,
    state: &Arc<Mutex<ClientStateStruct>>,
    sink: &RodioSink,
) -> Result<()> {
    print!("{}", "musicman❯ ".green().bold());
    stdout().flush().unwrap();
    let mut input = String::new();
    stdin().read_line(&mut input).unwrap();
    input = input.trim().to_string();
    let input = input
        .split_ascii_whitespace()
        .map(|s| s.to_string())
        .collect::<Vec<String>>();

    if input.is_empty() {
        return Ok(());
    }

    match input[0].as_str() {
        "replay" => handle_replay(&stream, &state),
        "pause" | "p" => handle_pause(&sink),
        "clear" => handle_clear(&state, &sink),
        "next" | "prev" => handle_next_prev(stream, state, input),
        "show" | "ls" => handle_show(state),
        "playlist" | "pl" => handle_playlist(stream, input, &state),
        "search" => handle_search(stream, input),
        "exit" => {
            println!("{}", "Exiting...".green());
            sink.lock().unwrap().stop();
            return Err(anyhow::anyhow!("Exit"));
        }
        cmd => {
            println!("{} {}", "Unknown command".red(), cmd.red().bold());
            print_help();
        }
    }
    Ok(())
}
