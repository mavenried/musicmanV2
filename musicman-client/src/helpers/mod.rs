use anyhow::Result;
use musicman_protocol::*;
use std::{
    io::{Read, Write},
    net::TcpStream,
};

mod playlist_response;
mod prompt;
mod search_response;

pub use playlist_response::*;
pub use prompt::*;
pub use search_response::*;

pub fn send_to_server(mut stream: &TcpStream, req: Request) {
    //println!("Sending: {req:?}");
    let req_bytes = bincode::serialize(&req).unwrap();
    let len = (req_bytes.len() as u32).to_be_bytes();
    stream.write_all(&len).unwrap();
    stream.write_all(&req_bytes).unwrap();
}
pub fn read_from_client(stream: &mut TcpStream) -> Result<Response> {
    let mut len_buf = [0u8; 4];
    stream.read_exact(&mut len_buf)?;
    let msg_len = u32::from_be_bytes(len_buf) as usize;

    let mut buf = vec![0u8; msg_len];
    stream.read_exact(&mut buf)?;
    let res: Response = bincode::deserialize(&buf)?;
    Ok(res)
}
