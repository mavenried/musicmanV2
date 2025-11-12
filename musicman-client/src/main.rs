use colored::Colorize;
use musicman_protocol::*;
use rodio::{OutputStream, Sink};
use std::io::Read;
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;

mod modules;
use modules::*;

type Stream = TcpStream;

fn server_interface(mut stream: Stream) {
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

fn main() {
    //get addr from args
    let args: Vec<String> = std::env::args().collect();

    let addr = if args.len() > 1 {
        args[1].clone()
    } else {
        "0.0.0.0:7878".to_string()
    };

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Arc::new(Sink::try_new(&stream_handle).unwrap());
    let stream = TcpStream::connect(&addr).unwrap();
    let state = Arc::new(Mutex::new(State {
        songid: None,
        queue: Vec::new(),
    }));

    let (stx, srx) = mpsc::channel::<Request>();
    let (utx, urx) = mpsc::channel::<Response>();
    let (ptx, prx) = mpsc::channel::<Request>();

    server_interface(stream.try_clone().unwrap());
    user_input(stream, state.clone(), sink.clone())
        .join()
        .unwrap();
}
