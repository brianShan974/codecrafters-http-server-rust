use std::{
    collections::HashMap,
    io::{BufRead, BufReader, Result},
    net::TcpStream,
};

use crate::{
    response::{self, Response, StatusCode},
    CRLF, DOUBLE_CRLF,
};

pub enum RequestType {
    Get,
}

pub enum ProtocolVersion {
    HTTP1P1,
}

pub struct RequestLine {
    request_type: RequestType,
    url: String,
    protocol: ProtocolVersion,
}

impl RequestLine {
    pub fn parse_line(line: &str) -> Option<Self> {
        let line: Vec<&str> = line.split(' ').collect();
        if line.len() != 3 || line[0] != "GET" || line[2] != "HTTP/1.1" {
            None
        } else {
            Some(Self {
                request_type: RequestType::Get,
                url: line[1].to_string(),
                protocol: ProtocolVersion::HTTP1P1,
            })
        }
    }
}

pub struct Request {
    line: RequestLine,
    headers: HashMap<String, String>,
    body: String,
}

impl Request {
    pub fn parse_string(request_string: String) -> Self {
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

        let body = if splitted_request.len() > 1 && !splitted_request[1].is_empty() {
            String::from(splitted_request[1])
        } else {
            String::new()
        };

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
        let buf_reader = BufReader::new(stream.try_clone()?);
        let request: Vec<_> = buf_reader
            .lines()
            .map(|result| result.unwrap())
            .take_while(|line| !line.is_empty())
            .collect();

        if request.len() == 1 {
            return Ok(Self::parse_string(request[0].clone() + CRLF));
        }

        let mut n = None;
        for (i, line) in request.iter().enumerate().skip(1) {
            if !line.contains(": ") {
                n = Some(i);
                break;
            }
        }

        if let Some(n) = n {
            let (line_and_headers, body) = request.split_at(n);
            let line_and_headers = line_and_headers.join(CRLF);
            let body = body.join("\n");
            let parse_string = [line_and_headers, body].join(DOUBLE_CRLF);
            Ok(Self::parse_string(parse_string))
        } else {
            Ok(Self::parse_string(request.join(CRLF)))
        }
    }

    pub fn construct_response(&self) -> Response {
        let url = self.line.url.as_str();
        if url == "/" {
            return Response::default();
        }

        let splitted_url: Vec<_> = url.split('/').skip(1).collect();
        let (head, length) = (splitted_url[0], splitted_url.len());
        if head == "echo" && length > 1 {
            let response_body = splitted_url[1].to_string();
            Response::construct_ok_with_body(response_body)
        } else if head == "user-agent" {
            let response_body = if let Some(user_agent) = self.read_field_from_header(head) {
                user_agent.to_string()
            } else {
                return Response::construct_not_found();
            };
            Response::construct_ok_with_body(response_body)
        } else {
            Response::construct_not_found()
        }
    }

    fn parse_headers(header_string: &str) -> HashMap<String, String> {
        let header_lines = header_string.split(CRLF);

        let mut headers = HashMap::new();

        for line in header_lines {
            if let Some((key, value)) = Self::parse_header_line(line) {
                headers.insert(key, value);
            }
        }

        headers
    }

    fn parse_header_line(header_line: &str) -> Option<(String, String)> {
        if !header_line.contains(": ") {
            return None;
        }

        let (key, value) = header_line.split_once(": ")?;

        Some((key.to_lowercase(), value.to_string()))
    }
}
