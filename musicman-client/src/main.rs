use colored::Colorize;
use musicman_protocol::{PlaylistRequest, Request};
use rodio::{OutputStream, Sink};
use std::io::{Write, stdin, stdout};
use std::sync::Arc;
use std::thread::sleep;
use std::time;

fn user_input() {
    loop {
        sleep(time::Duration::from_millis(10));
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
            continue;
        }

        match input[0].as_str() {
            "add" => {
                if input.len() > 1 {
                    println!("{}", "Adding song...".green());
                } else {
                    println!("{}", "add: Insufficient arguments".red());
                    println!("{}", "add <song name>".yellow().italic());
                }
            }
            "replay" => {
                println!("{}", "Replaying current song...".green());
            }
            "play" | "pause" | "p" => {
                println!("{}", "Toggling play/pause...".green());
            }
            "clear" => {
                println!("{}", "Clearing queue...".green());
            }
            "next" | "prev" => {
                if input.len() > 1 {
                    if let Ok(n) = input[1].parse::<usize>() {
                        if input[0] == "next" {
                            println!("{}", format!("Skipping {} song(s)...", n).green());
                        } else {
                            println!("{}", format!("Going back {} song(s)...", n).green());
                        }
                    } else if input[0] == "next" {
                        println!("{}", "Usage: next <+ve number>".yellow().italic());
                    } else {
                        println!("{}", "Usage: prev <+ve number>".yellow().italic());
                    }
                } else if input[0] == "next" {
                    println!("{}", "Skipping 1 song...".green());
                } else {
                    println!("{}", "Going back 1 song...".green());
                }
            }
            "exit" => {
                println!("{}", "Exiting...".green());
                break;
            }
            "show" | "ls" => {
                if input.len() > 1 && input[1] == "cp" {
                    println!("{}", "Showing current playlist...".green());
                } else {
                    println!("{}", "Showing all songs in queue...".green());
                }
            }
            "playlist" | "pl" => {
                if input.len() > 1 {
                    match input[1].as_str() {
                        "load" | "new" => {
                            let new = input[2..].join(" ").to_lowercase();
                            if input[1] == "load" {
                                println!("{}", format!("Loading playlist: {}", new).green());
                            } else {
                                println!("{}", format!("Creating new playlist: {}", new).green());
                            }
                        }
                        "ls" | "show" => {}
                        cmd => {
                            println!(
                                "{} {} {}",
                                "playlist: ".red(),
                                cmd.red().bold(),
                                " is not a valid command".red()
                            );
                            println!("{}", "Usage: playlist <add|new|show>".yellow());
                        }
                    }
                }
            }
            cmd => {
                println!("{} {}", "Unknown command".red(), cmd.red().bold());
                println!(
                    "{}",
                    "<add|clear|exit|next|p|playlist|prev|replay|show>"
                        .yellow()
                        .italic()
                );
            }
        }
    }
}
fn main() {
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Arc::new(Sink::try_new(&stream_handle).unwrap());
    std::thread::spawn(user_input)
        .join()
        .expect("Failed to join thread");
}

