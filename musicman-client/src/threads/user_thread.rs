use crate::{
    helpers::{self, send_to_server},
    player,
    types::*,
};
use colored::Colorize;
use musicman_protocol::*;
use std::{
    io::{Write, stdin, stdout},
    net::TcpStream,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
    thread, time,
};
use uuid::Uuid;

pub fn user_input(
    stream: TcpStream,
    state: Arc<Mutex<State>>,
    sink: RodioSink,
    urx: Receiver<UiRequest>,
    stx: Sender<UiResponse>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            thread::sleep(time::Duration::from_millis(100));
            if let Ok(req) = urx.try_recv() {
                match req {
                    UiRequest::Display(s) => println!("{}", s.yellow()),
                    UiRequest::Prompt { s, prompt } => {
                        println!("{}", s.blue());
                        print!("{}", (prompt + "❯ ").yellow().bold());
                        stdout().flush().unwrap();
                        let mut input = String::new();
                        stdin().read_line(&mut input).unwrap();
                        input = input.trim().to_string();
                        stx.send(UiResponse(input)).unwrap();
                    }
                    UiRequest::GetMeta(songs) => {
                        for song in songs {
                            send_to_server(&stream, Request::Meta { track_id: song });
                        }
                    }
                }
                continue;
            }
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
                        let songid = Uuid::parse_str(&input[1]).unwrap();
                        helpers::send_to_server(&stream, Request::Play { track_id: songid });
                        state.lock().unwrap().songid = Some(songid);
                    } else {
                        println!("{}", "add: Insufficient arguments".red());
                        println!("{}", "add <song name>".yellow().italic());
                    }
                }
                "replay" => {
                    println!("{}", "Replaying current song...".green());
                    let songid = state.lock().unwrap().songid;
                    if songid.is_none() {
                        println!("{}", "No song to replay".red());
                    } else {
                        helpers::send_to_server(
                            &stream,
                            Request::Play {
                                track_id: songid.unwrap().clone(),
                            },
                        );
                    }
                }
                "play" | "pause" | "p" => {
                    println!("{}", "Toggling play/pause...".green());
                    if sink.is_paused() {
                        sink.play()
                    } else {
                        sink.pause()
                    };
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
                                player::get_next_song(&state, n);
                            } else {
                                println!("{}", format!("Going back {} song(s)...", n).green());
                                player::get_prev_song(&state, n);
                            }
                        } else if input[0] == "next" {
                            println!("{}", "Usage: next <+ve number>".yellow().italic());
                        } else {
                            println!("{}", "Usage: prev <+ve number>".yellow().italic());
                        }
                    } else if input[0] == "next" {
                        println!("{}", "Skipping 1 song...".green());
                        player::get_next_song(&state, 1);
                    } else {
                        println!("{}", "Going back 1 song...".green());
                        player::get_prev_song(&state, 1);
                    }
                }
                "exit" => {
                    println!("{}", "Exiting...".green());
                    sink.stop();
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
                "meta" => {
                    if input.len() > 1 {
                        let songid = Uuid::parse_str(&input[1]).unwrap();
                        helpers::send_to_server(&stream, Request::Meta { track_id: songid });
                    } else {
                        println!("{}", "add: Insufficient arguments".red());
                        println!("{}", "add <song name>".yellow().italic());
                    }
                }
                "search" => {
                    if input.len() > 1 {
                        match input[1].as_str() {
                            "artist" | "a" => {
                                let artist = input[2..].join(" ").to_lowercase();
                                helpers::send_to_server(
                                    &stream,
                                    Request::Search(SearchType::ByArtist(artist)),
                                );
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
