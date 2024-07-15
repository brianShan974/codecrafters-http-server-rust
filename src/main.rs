use std::{
    io::{BufRead, BufReader, Write},
    net::{TcpListener, TcpStream},
};

fn handle_connection(stream: &mut TcpStream) -> Result<(), std::io::Error> {
    println!("handle_connection is called.");
    let mut buf_reader = BufReader::new(stream.try_clone()?);
    let mut request = String::new();
    buf_reader.read_line(&mut request)?;
    println!("request is read to string.");

    let mut request_line = request.split("\r\n").next().unwrap().split(' ');
    let _request_type = request_line.next();
    let request_path = request_line.next().unwrap();

    let response = if request_path != "/" {
        "HTTP/1.1 200 OK\r\n\r\n"
    } else {
        "HTTP/1.1 404 Not Found\r\n\r\n"
    }
    .as_bytes();
    // println!("{}", String::from_utf8_lossy(response));

    println!("Preparing to write the response.");
    stream.write_all(response)?;
    println!("Responded.");
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
