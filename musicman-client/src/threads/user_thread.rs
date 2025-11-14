use crate::{helpers, types::*};
use colored::Colorize;
use reedline::{FileBackedHistory, Reedline, Signal};
use std::{
    net::TcpStream,
    path::PathBuf,
    sync::{
        Arc, Mutex,
        mpsc::{Receiver, Sender},
    },
    thread::{self, sleep},
    time,
};

pub fn user_input(
    stream: TcpStream,
    state: Arc<Mutex<ClientStateStruct>>,
    sink: RodioSink,
    urx: Receiver<UiRequest>,
    stx: Sender<UiResponse>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        let mut hist_path = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("musicman");
        hist_path.push("musicman_history");
        let history = Box::new(FileBackedHistory::with_file(1000, hist_path).unwrap());
        let highlighter = Box::new(MusicmanHighlighter::new());

        let mut editor = Reedline::create()
            .with_history(history)
            .with_highlighter(highlighter);
        let mut prompt_editor = Reedline::create();

        loop {
            sleep(time::Duration::from_millis(100));

            if let Ok(req) = urx.try_recv() {
                match req {
                    UiRequest::Display(s) => {
                        println!("{}", s.yellow());
                    }

                    UiRequest::Prompt { s, prompt } => {
                        println!("{}", s.blue());

                        let p = MusicmanPrompt {
                            left: format!("{}❯ ", prompt).yellow().bold().to_string(),
                        };

                        match prompt_editor.read_line(&p) {
                            Ok(Signal::Success(line)) => {
                                stx.send(UiResponse(line.trim().to_string())).ok();
                            }
                            Ok(Signal::CtrlC) => {
                                stx.send(UiResponse("".into())).ok();
                            }
                            Ok(Signal::CtrlD) => break,
                            Err(_) => break,
                        }
                    }

                    UiRequest::Shutdown => {
                        println!("Shutting down...");
                        break;
                    }
                }
            } else {
                if let Err(_) = helpers::musicman_prompt(&stream, &state, &sink, &mut editor) {
                    break;
                }
            }
        }
    })
}

