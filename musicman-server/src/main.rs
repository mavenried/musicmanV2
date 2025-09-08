use bincode;
use musicman_protocol::{Request, Response};
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener; // <- your shared crate

use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::FormatOptions;
use symphonia::core::io::MediaSourceStream;
use symphonia::core::meta::MetadataOptions;
use symphonia::default::get_probe;

use std::fs::File;
use std::sync::Arc;

mod utils;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let listener = TcpListener::bind("0.0.0.0:4000").await?;
    tracing::info!("Server listening on 0.0.0.0:4000");

    loop {
        let (mut socket, addr) = listener.accept().await?;
        tracing::info!("New client: {:?}", addr);

        tokio::spawn(async move {
            if let Err(e) = handle_client(&mut socket).await {
                tracing::error!("Error with client {:?}: {:?}", addr, e);
            }
        });
    }
}

async fn handle_client(socket: &mut tokio::net::TcpStream) -> anyhow::Result<()> {
    let mut buf = vec![0u8; 4096];

    loop {
        let n = socket.read(&mut buf).await?;
        if n == 0 {
            break; // client closed
        }

        // Deserialize request
        let request: Request = bincode::deserialize(&buf[..n])?;

        match request {
            Request::Play { track_id } => {
                tracing::info!("Client requested track {}", track_id);
            }

            _ => {
                tracing::warn!("Unsupported request: {:?}", request);
            }
        }
    }

    Ok(())
}
