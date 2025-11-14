use crate::types::ClientState;
use crate::types::GetReturn;

pub fn get_prev_song(state: &ClientState, n: usize) -> GetReturn {
    let mut state = state.lock().unwrap();

    if state.queue.is_empty() {
        return GetReturn::QueueEmpty;
    }

    if state.current_song.is_none() {
        let current_playing = state.queue[0].clone();
        state.current_idx = 0;

        state.current_song = Some(current_playing);
        return GetReturn::Ok;
    }

    let len = state.queue.len();
    let new_index = (state.current_idx + len - (n % len)) % len;

    state.current_idx = new_index;
    let current_playing = state.queue[new_index].clone();
    state.current_song = Some(current_playing);
    GetReturn::Ok
}

pub fn get_next_song(state: &ClientState, n: usize) -> GetReturn {
    let mut state = state.lock().unwrap();

    if state.queue.is_empty() {
        return GetReturn::QueueEmpty;
    }

    if state.current_song.is_none() {
        let current_playing = state.queue[0].clone();
        state.current_idx = 0;

        state.current_song = Some(current_playing);
        return GetReturn::Ok;
    }

    let new_index = (state.current_idx + n) % state.queue.len();

    state.current_idx = new_index;
    let current_playing = state.queue[new_index].clone();
    state.current_song = Some(current_playing);
    GetReturn::Ok
}
