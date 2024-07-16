mod request;
mod response;

use std::{
    io::{Result, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use crate::request::Request;

pub const CRLF: &str = "\r\n";
pub const DOUBLE_CRLF: &str = "\r\n\r\n";

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    let request = Request::read_full_request(&mut stream)?;
    let response = request.construct_response();
    stream.write_all(response.get_response_string().as_bytes())?;
    Ok(())
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let handle = thread::spawn(move || {
                    println!("accepted new connection");
                    handle_connection(stream).unwrap();
                });
                handle.join().unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
