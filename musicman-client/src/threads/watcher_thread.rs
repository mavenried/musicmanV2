use crate::{helpers::send_to_server, player, types::*};
use musicman_protocol::*;
use std::{net::TcpStream, thread, thread::sleep, time::Duration};

pub fn watcher_thread(
    stream: TcpStream,
    sink: RodioSink,
    state: ClientState,
    player_state: PlayerState,
) {
    thread::spawn(move || {
        loop {
            let mut ps = match player_state.lock() {
                Ok(p) => p,
                Err(_) => {
                    sleep(Duration::from_millis(500));
                    continue;
                }
            };

            if ps.waiting_for_header {
                drop(ps);
                sleep(Duration::from_millis(500));
                continue;
            }

            let empty = match sink.lock() {
                Ok(s) => s.empty(),
                Err(_) => {
                    sleep(Duration::from_millis(500));
                    continue;
                }
            };

            if !empty {
                drop(ps);
                sleep(Duration::from_millis(500));
                continue;
            }

            ps.waiting_for_header = true;
            drop(ps);

            if player::get_next_song(&state, 1) == GetReturn::QueueEmpty {
                continue;
            }

            if let Ok(st) = state.lock() {
                let track_id = st.current_song.clone().unwrap().id.clone();
                send_to_server(&stream, Request::Play { track_id });
            }

            sleep(Duration::from_millis(500));
        }
    });
}
