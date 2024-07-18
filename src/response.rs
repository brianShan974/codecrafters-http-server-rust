use std::collections::HashMap;

use crate::{request::HTTP_PROTOCOL, response, CRLF};

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

impl ToString for ContentType {
    fn to_string(&self) -> String {
        match self {
            Self::PlainText => "text/plain".to_string(),
            Self::OctetStream => "application/octet-stream".to_string(),
        }
    }
}

#[derive(Debug, Default)]
pub struct Response {
    status: StatusCode,
    headers: HashMap<String, String>,
    response_body: String,
}

impl Response {
    pub fn new(
        status: StatusCode,
        headers: HashMap<String, String>,
        response_body: String,
    ) -> Self {
        Self {
            status,
            headers,
            response_body,
        }
    }

    pub fn construct_ok_with_body(
        response_body: String,
        headers: Option<HashMap<String, String>>,
    ) -> Self {
        let headers = if let Some(headers) = headers {
            headers
        } else {
            HashMap::new()
        };
        Self {
            status: StatusCode::Ok,
            headers,
            response_body,
        }
    }

    pub fn construct_created() -> Self {
        Self {
            status: StatusCode::Created,
            headers: HashMap::new(),
            response_body: String::new(),
        }
    }

    pub fn construct_not_found() -> Self {
        Self {
            status: StatusCode::NotFound,
            headers: HashMap::new(),
            response_body: String::new(),
        }
    }

    pub fn get_response_string(&self) -> String {
        let mut response = String::new();

        self.push_status_line_string(&mut response);

        if let StatusCode::Created = self.status {
            response.push_str(CRLF);
            return response;
        }

        self.push_header_string(&mut response);

        self.push_body_string(&mut response);

        response
    }

    fn push_status_line_string(&self, dest: &mut String) {
        dest.push_str(HTTP_PROTOCOL);
        dest.push_str(match self.status {
            StatusCode::Ok => " 200 OK",
            StatusCode::NotFound => " 404 Not Found",
            StatusCode::Created => " 201 Created",
        });
        dest.push_str(CRLF);
    }

    fn push_header_string(&self, dest: &mut String) {
        for (key, val) in &self.headers {
            dest.push_str(key);
            dest.push_str(": ");
            dest.push_str(val);
            dest.push_str(CRLF);
        }
        dest.push_str(CRLF);
    }

    fn push_body_string(&self, dest: &mut String) {
        if !self.response_body.is_empty() {
            dest.push_str(&self.response_body);
            dest.push_str(CRLF);
        }
    }
}
