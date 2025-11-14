use musicman_protocols::*;
use rodio::{OutputStream, Sink};
use std::net::TcpStream;
use std::process::exit;
use std::sync::mpsc;
use std::sync::{Arc, Mutex};

mod handlers;
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
        "0.0.0.0:4000".to_string()
    };

    let (_stream, stream_handle) = OutputStream::try_default().unwrap();

    let sink = Arc::new(Mutex::new(Sink::try_new(&stream_handle).unwrap()));
    let Ok(stream) = TcpStream::connect(&addr) else {
        println!("Connection Refused");
        exit(1);
    };

    let state = Arc::new(Mutex::new(ClientStateStruct {
        queue: Vec::new(),
        current_song: None,
        current_idx: 0,
    }));
    let player_state = Arc::new(Mutex::new(PlayerStateStruct {
        channels: 2,
        sample_rate: 48000,
        current_id: None,
        waiting_for_header: false,
    }));

    let (stx, srx) = mpsc::channel::<UiResponse>();
    let (utx, urx) = mpsc::channel::<UiRequest>();
    let (ptx, prx) = mpsc::channel::<Response>();

    threads::server_interface(stream.try_clone().unwrap(), state.clone(), ptx, utx, srx);
    threads::player(prx, sink.clone(), player_state.clone());
    threads::watcher_thread(
        stream.try_clone().unwrap(),
        sink.clone(),
        state.clone(),
        player_state.clone(),
    );
    threads::user_input(stream, state, sink, urx, stx)
        .join()
        .unwrap();
}
