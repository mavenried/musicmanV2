use crate::{helpers, player, types::*};
use colored::Colorize;
use musicman_protocol::*;
use rodio::{OutputStream, Sink};
use std::{
    io::{Read, Write, stdin, stdout},
    net::TcpStream,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
    thread, time,
};
use uuid::Uuid;

type Stream = TcpStream;

pub fn user_input(
    stream: TcpStream,
    state: Arc<Mutex<State>>,
    sink: RodioSink,
) -> thread::JoinHandle<()> {
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

pub fn server_interface(mut stream: Stream, ptx: Sender<Response>) {
    thread::spawn(move || {
        let mut reader = std::io::BufReader::new(&mut stream);
        let mut buffer = vec![0; 4096];

        loop {
            let n = reader.read(&mut buffer);

            match n {
                Ok(0) => {
                    // Connection closed
                    println!("{}", "Error: Connection closed by server".red());
                    break;
                }
                Ok(_) => {
                    if let Ok(response) = bincode::deserialize::<Response>(&buffer) {
                        match response {
                            Response::Meta(songmeta) => {
                                let mins = songmeta.duration / 60;
                                let secs = songmeta.duration % 60;
                                println!("Title: {}", songmeta.title);
                                println!("Artist(s): {}", songmeta.artists.join(", "));
                                println!("Duration: {}m {}s", mins, secs);
                            }
                            Response::SongChunk { .. }
                            | Response::SongHeader { .. }
                            | Response::EndOfStream { .. } => {
                                ptx.send(response).unwrap();
                            }

                            _ => {
                                println!("Received response: {:?}", response);
                            }
                        }
                    } else {
                        println!("{}", "Error: Failed to deserialize response".red());
                    }
                }
                Err(e) => {
                    println!("Error reading from server: {}", e);
                    break;
                }
            }
        }
    });
}

pub fn player(prx: Receiver<Response>, sink: Arc<Sink>) {
    thread::spawn(move || {
        let mut samples: Vec<i16> = Vec::new();

        // Collect all audio chunks first
        while let Ok(msg) = prx.recv() {
            match msg {
                Response::SongChunk { data, .. } => {
                    samples.extend_from_slice(&data);
                }
                Response::EndOfStream { track_id: _ } => {
                    break;
                }
                Response::Error { message } => {
                    eprintln!("Server error: {}", message);
                    return;
                }
                _ => {}
            }
        }

        if samples.is_empty() {
            eprintln!("No audio data received.");
            return;
        }

        // Playback once fully buffered
        let (_stream, stream_handle) = match OutputStream::try_default() {
            Ok(s) => s,
            Err(e) => {
                eprintln!("Audio output error: {}", e);
                return;
            }
        };

        // convert Vec<i16> -> rodio Source
        let source = rodio::buffer::SamplesBuffer::new(2, 44100, samples);
        sink.append(source);
        sink.sleep_until_end();
    });
}
