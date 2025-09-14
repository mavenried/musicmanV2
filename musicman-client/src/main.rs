use colored::Colorize;
use musicman_protocol::{PlaylistRequest, Request, Response};
use rodio::{OutputStream, Sink};
use std::io::{BufRead, Read, Write, stdin, stdout};
use std::net::{TcpListener, TcpStream};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time;

mod modules;
use modules::*;

type Stream = Arc<Mutex<TcpStream>>;

fn server_interface(listener: Stream) {
    thread::spawn(|| {
        loop {
            thread::sleep(time::Duration::from_millis(1000));
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
    let stream = Arc::new(Mutex::new(TcpStream::connect(&addr).unwrap()));
    let state = Arc::new(Mutex::new(State {
        songid: String::new(),
        queue: Vec::new(),
    }));

    let (stx, srx) = mpsc::channel::<Request>();
    let (utx, urx) = mpsc::channel::<Response>();
    let (ptx, prx) = mpsc::channel::<Request>();

    server_interface(stream.clone());
    user_input(stream, state.clone()).join().unwrap();
}
