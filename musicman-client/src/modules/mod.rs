use colored::Colorize;
use musicman_protocol::Request;
use std::io::{Write, stdin, stdout};
use std::net::TcpStream;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

type Stream = Arc<Mutex<TcpStream>>;

pub struct State {
    pub songid: String,
    pub queue: Vec<String>,
}

fn send_to_server(stream: &Stream, req: Request) {
    let mut stream = stream.lock().unwrap();
    let req_bytes = bincode::serialize(&req).unwrap();
    let req_bytes = req_bytes.as_slice();
    stream.write_all(req_bytes).unwrap();
}

pub fn user_input(stream: Stream, state: Arc<Mutex<State>>) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(time::Duration::from_millis(10));
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
                        let songid = input[1..].join(" ");
                        send_to_server(
                            &stream,
                            Request::Play {
                                track_id: songid.clone(),
                            },
                        );
                        state.lock().unwrap().songid = songid;
                    } else {
                        println!("{}", "add: Insufficient arguments".red());
                        println!("{}", "add <song name>".yellow().italic());
                    }
                }
                "replay" => {
                    println!("{}", "Replaying current song...".green());
                    let songid = state.lock().unwrap().songid.clone();
                    if songid.is_empty() {
                        println!("{}", "No song to replay".red());
                    } else {
                        send_to_server(
                            &stream,
                            Request::Play {
                                track_id: songid.clone(),
                            },
                        );
                    }
                }
                "play" | "pause" | "p" => {
                    println!("{}", "Toggling play/pause...".green());
                }
                "clear" => {
                    println!("{}", "Clearing queue...".green());
                    state.lock().unwrap().queue.clear();
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
                                    println!(
                                        "{}",
                                        format!("Creating new playlist: {}", new).green()
                                    );
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
    })
}
