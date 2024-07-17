use std::default;

use crate::{response, CRLF};

#[derive(Debug, Default)]
pub enum StatusCode {
    #[default]
    Ok,
    NotFound,
    Created,
}

#[derive(Debug)]
pub enum ContentType {
    PlainText,
    OctetStream,
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

    pub fn construct_ok_with_body(response_body: String, content_type: ContentType) -> Self {
        Self {
            status: StatusCode::Ok,
            content_type: Some(content_type),
            content_length: Some(response_body.len()),
            response_body,
        }
    }

    pub fn construct_created() -> Self {
        Self {
            status: StatusCode::Created,
            content_type: None,
            content_length: None,
            response_body: String::new(),
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
            StatusCode::Created => "HTTP/1.1 201 Created",
        });
        response.push_str(CRLF);

        if let StatusCode::Created = self.status {
            return response;
        }

        if let Some(content_type) = &self.content_type {
            response.push_str("Content-Type: ");
            response.push_str(match content_type {
                ContentType::PlainText => "text/plain",
                ContentType::OctetStream => "application/octet-stream",
            });
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
