use musicman_protocol::*;
use rodio::{OutputStream, Sink};
use std::net::TcpStream;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

mod helpers;
mod player;
mod threads;
mod types;
use types::*;

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

    let (stx, srx) = mpsc::channel::<UiResponse>();
    let (utx, urx) = mpsc::channel::<UiRequest>();
    let (ptx, prx) = mpsc::channel::<Response>();

    threads::server_interface(stream.try_clone().unwrap(), ptx, utx, srx);

    threads::player(prx, sink.clone());
    threads::user_input(stream, state.clone(), sink.clone(), urx, stx)
        .join()
        .unwrap();
}
