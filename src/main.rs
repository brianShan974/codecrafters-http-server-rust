mod request;
mod response;

use clap::Parser;

use std::{
    io::{Result, Write},
    net::{TcpListener, TcpStream},
    thread,
};

use lazy_static::lazy_static;

use crate::request::Request;

pub const CRLF: &str = "\r\n";
pub const DOUBLE_CRLF: &str = "\r\n\r\n";

lazy_static! {
    static ref PATH: String = Args::parse().directory;
}

fn handle_connection(mut stream: TcpStream) -> Result<()> {
    // println!("handle_connection called.");
    let request = Request::read_full_request(&mut stream)?;
    // println!("request read into struct.");
    let response = request.construct_response();
    // println!("response constructed.");
    let response_string = response.get_response_string();
    // println!("response: {}", response_string);
    stream.write_all(response_string.as_bytes())?;
    // println!("response written.");
    Ok(())
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for (i, stream) in listener.incoming().enumerate() {
        // match stream {
        //     Ok(stream) => {
        //         let handle = thread::spawn(|| {
        //             println!("accepted new connection");
        //             handle_connection(stream).unwrap();
        //         });
        //         handle.join().unwrap();
        //     }
        //     Err(e) => {
        //         println!("error: {}", e);
        //     }
        // }
        let stream = stream.unwrap();

        let _handle = thread::spawn(move || {
            println!("thread {} accepted new connection", i + 1);
            handle_connection(stream).unwrap();
        });
    }
}

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    directory: String,
}
