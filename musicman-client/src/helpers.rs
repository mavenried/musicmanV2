use musicman_protocol::Request;
use std::io::Write;
use std::net::TcpStream;

pub fn send_to_server(mut stream: &TcpStream, req: Request) {
    let req_bytes = bincode::serialize(&req).unwrap();
    let req_bytes = req_bytes.as_slice();
    stream.write_all(req_bytes).unwrap();
}

