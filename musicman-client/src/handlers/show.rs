use crate::types::{ClientState, PlayingDisplay, SongTable};
use colored::Colorize;
use tabled::{
    Table,
    settings::{Remove, Style, object::Columns},
};

pub fn handle_show(state: &ClientState) {
    let songs;
    let maybe_current_song;
    if let Ok(s) = state.lock() {
        songs = s.queue.clone();
        maybe_current_song = s.current_song.clone();
    } else {
        return;
    }
    if songs.is_empty() {
        println!("{}", "Queue is Empty.".yellow());
        return;
    }
    let songtable = Table::new(songs.iter().map(|sm| {
        let mins = sm.duration / 60;
        let secs = sm.duration % 60;
        SongTable {
            playing: PlayingDisplay(if let Some(song) = maybe_current_song.clone() {
                song.id == sm.id
            } else {
                false
            }),
            id: 0,
            title: sm.title.clone(),
            artists: sm.artists.join(", "),
            duration: format!("{mins}m{secs}s"),
        }
    }))
    .with(Style::rounded())
    .with(Remove::column(Columns::one(0)))
    .to_string();

    println!("{}", songtable.yellow());
}
