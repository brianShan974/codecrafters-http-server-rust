use std::default;

use crate::{response, CRLF};

#[derive(Debug, Default)]
pub enum StatusCode {
    #[default]
    Ok,
    NotFound,
}

#[derive(Debug)]
pub enum ContentType {
    PlainText,
}

#[derive(Debug, Default)]
pub struct Response {
    status: StatusCode,
    content_type: Option<ContentType>,
    content_length: Option<usize>,
    response_body: String,
}

impl Response {
    pub fn new(
        status: StatusCode,
        content_type: Option<ContentType>,
        content_length: Option<usize>,
        response_body: String,
    ) -> Self {
        Self {
            status,
            content_type,
            content_length,
            response_body,
        }
    }

    pub fn construct_ok_with_body(response_body: String) -> Self {
        Self {
            status: StatusCode::Ok,
            content_type: Some(ContentType::PlainText),
            content_length: Some(response_body.len()),
            response_body,
        }
    }

    pub fn construct_not_found() -> Self {
        Self {
            status: StatusCode::NotFound,
            content_type: None,
            content_length: None,
            response_body: String::new(),
        }
    }

    pub fn set_response_body(&mut self, body: String) {
        self.response_body = body;
    }

    pub fn get_response_string(&self) -> String {
        let mut response = String::new();

        response.push_str(match self.status {
            StatusCode::Ok => "HTTP/1.1 200 OK",
            StatusCode::NotFound => "HTTP/1.1 404 Not Found",
        });
        response.push_str(CRLF);

        if let Some(ContentType::PlainText) = self.content_type {
            response.push_str("Content-Type: text/plain");
            response.push_str(CRLF);

            if let Some(size) = self.content_length {
                response.push_str(&format!("Content-Length: {}", size));
                response.push_str(CRLF);
            }
        }
        response.push_str(CRLF);

        if !self.response_body.is_empty() {
            response.push_str(&self.response_body);
            response.push_str(CRLF);
        }

        response
    }
}
