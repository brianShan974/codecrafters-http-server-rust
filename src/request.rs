use std::{
    collections::HashMap,
    fs,
    io::{prelude::*, BufReader, Result},
    net::TcpStream,
};

use flate2::{write::GzEncoder, Compression};

use crate::{
    response::{ContentType, Response},
    CRLF, DOUBLE_CRLF, PATH,
};

pub const HTTP_PROTOCOL: &str = "HTTP/1.1";
pub const AVAILABLE_ENCODINGS: [&str; 1] = ["gzip"];

#[derive(Debug)]
pub enum RequestType {
    Get,
    Post,
}

#[derive(Debug)]
pub enum ProtocolVersion {
    HTTP1P1,
}

#[derive(Debug)]
pub struct RequestLine {
    request_type: RequestType,
    url: String,
    protocol: ProtocolVersion,
}

impl RequestLine {
    pub fn parse_line(line: &str) -> Option<Self> {
        let line: Vec<&str> = line.split(' ').collect();
        if line.len() != 3 || line[2] != HTTP_PROTOCOL {
            None
        } else {
            let request_type = if line[0] == "GET" {
                RequestType::Get
            } else if line[0] == "POST" {
                RequestType::Post
            } else {
                return None;
            };
            Some(Self {
                request_type,
                url: line[1].to_string(),
                protocol: ProtocolVersion::HTTP1P1,
            })
        }
    }
}

#[derive(Debug)]
pub struct Request {
    line: RequestLine,
    headers: HashMap<String, String>,
    body: String,
}

impl Request {
    pub fn parse_string_with_body(request_string: String, body: String) -> Self {
        let splitted_request: Vec<&str> = request_string.as_str().split(DOUBLE_CRLF).collect();
        let line_and_headers: Vec<&str> = splitted_request[0].split(CRLF).collect();

        let line = RequestLine::parse_line(line_and_headers[0]).unwrap();

        let mut headers = HashMap::new();
        if line_and_headers.len() > 1 {
            for line in line_and_headers.into_iter().skip(1) {
                if let Some((key, value)) = Self::parse_header_line(line) {
                    headers.insert(key, value);
                } else {
                    break;
                }
            }
        }

        // let body = if splitted_request.len() > 1 && !splitted_request[1].is_empty() {
        //     String::from(splitted_request[1])
        // } else {
        //     String::new()
        // };

        Self {
            line,
            headers,
            body,
        }
    }

    pub fn read_field_from_header(&self, field: &str) -> Option<&String> {
        let key = field.to_lowercase();
        self.headers.get(&key)
    }

    pub fn read_full_request(stream: &mut TcpStream) -> Result<Self> {
        // println!("read_full_request called.");
        let mut buf_reader = BufReader::new(stream.try_clone()?);
        // println!("stream cloned into buf_reader.");
        let mut has_content: bool = false;
        let mut content_length: usize = 0;
        let mut should_start_reading_body: bool = false;

        let mut request: Vec<String> = Vec::new();

        let mut line = String::new();
        loop {
            if !should_start_reading_body {
                buf_reader.read_line(&mut line)?;
            } else {
                // println!("I should start reading content body.");
                let mut buffer = vec![0; content_length];
                buf_reader.read_exact(&mut buffer)?;
                line.extend(buffer.iter().map(|c| *c as char));
                request.push(line.trim().to_string());
                break;
            }
            let trimmed_line = line.trim();
            if !trimmed_line.is_empty() {
                // println!("The line is not empty so I'm pushing it into the vector.");
                request.push(trimmed_line.to_string());
            } else if has_content && content_length != 0 {
                // println!("The line is empty itself, but it still has content so I'm pushing later lines into the vector.");
                has_content = false;
                should_start_reading_body = true;
                continue;
            } else {
                // println!("The line is empty, and there is no content left so I'm quitting.");
                break;
            }
            if line.starts_with("Content-Length") {
                let num_str = trimmed_line.split(": ").nth(1).unwrap();
                content_length = num_str.parse().expect("Invalid format of content length.");
                has_content = true;
            }
            // println!();
            line.clear();
        }

        let body = if content_length > 0 {
            let last = request.len() - 1;
            request[last].clone()
        } else {
            String::new()
        };

        // println!("request read as lines: {:?}", request);

        if request.len() == 1 {
            return Ok(Self::parse_string_with_body(
                request[0].clone() + CRLF,
                body,
            ));
        }

        let request = Self::parse_string_with_body(request.join(CRLF), body);

        Ok(request)
    }

    pub fn construct_response(&self) -> Response {
        let url = self.line.url.as_str();
        if url == "/" {
            return Response::default();
        }

        let splitted_url: Vec<_> = url.split('/').skip(1).collect();
        let (head, length) = (splitted_url[0], splitted_url.len());
        if head == "echo" && length > 1 {
            self.construct_echo_response(splitted_url)
        } else if head == "user-agent" {
            self.construct_user_agent_response(splitted_url)
        } else if head == "files" {
            self.construct_file_response(splitted_url)
        } else {
            Response::construct_not_found()
        }
    }

    fn construct_echo_response(&self, splitted_url: Vec<&str>) -> Response {
        let mut response_body = splitted_url[1].to_string();
        let mut headers = self.construct_headers(ContentType::PlainText, &response_body);
        if let Some(encodings) = self.headers.get("accept-encoding") {
            let encoding = self.find_available_encoding(encodings);
            if let Some(encoding) = encoding {
                headers.insert("Content-Encoding".to_string(), encoding);
            }
        }
        if let Some("gzip") = headers.get("Content-Encoding").map(|s| s.as_str()) {
            let result = Self::compress_with_gzip(&response_body);
            let result = unsafe { String::from_utf8_unchecked(result) };
            response_body.clear();
            response_body.push_str(&result);
            *headers.get_mut("Content-Length").unwrap() = response_body.len().to_string();
        }
        Response::construct_ok_with_body(response_body, Some(headers))
    }

    fn construct_user_agent_response(&self, splitted_url: Vec<&str>) -> Response {
        let response_body = if let Some(user_agent) = self.read_field_from_header(splitted_url[0]) {
            user_agent.to_string()
        } else {
            return Response::construct_not_found();
        };
        let headers = self.construct_headers(ContentType::PlainText, &response_body);
        Response::construct_ok_with_body(response_body, Some(headers))
    }

    fn construct_file_response(&self, splitted_url: Vec<&str>) -> Response {
        if splitted_url.len() <= 1 {
            return Response::construct_not_found();
        }
        let path = splitted_url[1];
        let path = if !PATH.ends_with('/') {
            PATH.to_string() + path
        } else {
            PATH.to_string() + "/" + path
        };
        match self.line.request_type {
            RequestType::Get => {
                let file_string = if let Ok(file_string) = Self::read_from_file(&path) {
                    file_string
                } else {
                    return Response::construct_not_found();
                };
                let headers = self.construct_headers(ContentType::OctetStream, &file_string);
                Response::construct_ok_with_body(file_string, Some(headers))
            }
            RequestType::Post => {
                if Self::create_file(&path, self.body.clone()).is_ok() {
                    Response::construct_created()
                } else {
                    Response::construct_not_found()
                }
            }
        }
    }

    fn construct_headers(
        &self,
        content_type: ContentType,
        response_body: &str,
    ) -> HashMap<String, String> {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), content_type.to_string());
        headers.insert(
            "Content-Length".to_string(),
            response_body.len().to_string(),
        );
        headers
    }

    fn find_available_encoding(&self, encodings: &str) -> Option<String> {
        encodings
            .split(", ")
            .filter(|encoding| AVAILABLE_ENCODINGS.contains(encoding))
            .map(|encoding| encoding.to_string())
            .next()
    }

    fn parse_header_line(header_line: &str) -> Option<(String, String)> {
        if !header_line.contains(": ") {
            return None;
        }

        let (key, value) = header_line.split_once(": ")?;

        Some((key.to_lowercase(), value.to_string()))
    }

    fn read_from_file(path: &str) -> Result<String> {
        fs::read_to_string(path)
    }

    fn create_file(path: &str, content: String) -> Result<()> {
        fs::write(path, content.clone())
    }

    fn compress_with_gzip(input: &str) -> Vec<u8> {
        println!("before encoding: {input}");
        let buffer: Vec<u8> = Vec::new();
        let compression_level = Compression::default();
        let mut encoder = GzEncoder::new(buffer, compression_level);
        encoder.write_all(input.as_bytes()).unwrap();
        let result = encoder.finish().unwrap();
        println!("encoded: {}", unsafe {
            String::from_utf8_unchecked(result.clone())
        });
        result
    }
}
