use crate::helpers;
use crate::types::*;
use colored::Colorize;
use std::{
    io::{Write, stdin, stdout},
    net::TcpStream,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
    thread::{self, sleep},
    time::{self},
};

pub fn user_input(
    stream: TcpStream,
    state: Arc<Mutex<ClientStateStruct>>,
    sink: RodioSink,
    urx: Receiver<UiRequest>,
    stx: Sender<UiResponse>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        loop {
            sleep(time::Duration::from_millis(100));
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
                    UiRequest::Shutdown => {
                        println!("Shutting down...");
                        break;
                    }
                }
            } else {
                if let Err(_) = helpers::handle_prompt(&stream, &state, &sink) {
                    break;
                }
            }
        }
    })
}
