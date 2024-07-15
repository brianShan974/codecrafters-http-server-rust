mod response;

use std::{
    io::{BufRead, BufReader, Result, Write},
    net::{TcpListener, TcpStream},
};

use crate::response::{ContentType, Response, StatusCode};

pub const CRLF: &str = "\r\n";

fn get_request(stream: &mut TcpStream) -> Result<String> {
    let mut buf_reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    buf_reader.read_line(&mut request)?;

    Ok(request)
}

fn get_url_from_request(request: &str) -> String {
    let request_line = request.split(CRLF).next().unwrap();
    let mut splitted_request_line = request_line.split(' ');

    let _request_type = splitted_request_line.next();
    let request_path = splitted_request_line.next().unwrap();

    request_path.to_string()
}

fn parse_url_for_response(url: &str) -> Option<(String, ContentType, usize)> {
    let mut splitted_url = url.split('/').skip(1);
    // println!("{splitted_url:?}");

    let echo = splitted_url.next().unwrap();
    if echo != "echo" {
        return None;
    }
    let body = splitted_url.next()?;

    Some((body.to_string(), ContentType::PlainText, body.len()))
}

fn handle_connection(stream: &mut TcpStream) -> Result<()> {
    let request = get_request(stream)?;
    let url = get_url_from_request(&request);

    let response = {
        let response = if let Some((response_body, content_type, content_length)) =
            parse_url_for_response(&url)
        {
            Response::new(
                StatusCode::Ok,
                Some(content_type),
                Some(content_length),
                response_body,
            )
        } else {
            Response::new(StatusCode::NotFound, None, None, String::new())
        };
        response.get_response_string()
    };

    stream.write_all(response.as_bytes())?;

    Ok(())
}

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    // println!("Logs from your program will appear here!");

    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");
                handle_connection(&mut stream).unwrap();
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
