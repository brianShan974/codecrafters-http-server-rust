use std::{
    error::Error,
    io::{Read, Write},
    net::{TcpListener, TcpStream},
};

fn handle_connection(stream: &mut TcpStream) -> Result<(), Box<dyn Error>> {
    let mut request = String::new();
    stream.read_to_string(&mut request)?;

    let mut request_line = request.split("\r\n").next().unwrap().split(' ');
    let _request_type = request_line.next();
    let request_path = request_line.next().unwrap();

    let response = if request_path != "/" {
        "HTTP/1.1 200 OK\r\n\r\n"
    } else {
        "HTTP/1.1 404 Not Found\r\n\r\n"
    };
    // println!("{}", String::from_utf8_lossy(response));

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
