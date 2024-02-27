use std::{collections::HashMap, net::TcpStream, io::Write};



pub enum HttpBody {
    String(String),
    Bytes(Vec<u8>)
}

pub struct HttpResponse {
    status: HttpStatus,
    content: Box<Vec<u8>>,
    headers: HashMap<String,String>
}


impl HttpResponse {

    pub fn new(status:HttpStatus, content:impl Into<Vec<u8>>) -> HttpResponse {
        HttpResponse::new_with_headers(status, HashMap::new(), content)
    }

    pub fn ok_from<T: Into<Vec<u8>>>(content: T) -> HttpResponse {
        HttpResponse::new(HttpStatus::Ok, content.into())
    }

    pub fn ok() -> HttpResponse {
        HttpResponse::new(HttpStatus::Ok, vec![])
    }

    pub fn not_found() -> HttpResponse {
        HttpResponse::new(HttpStatus::NotFound, vec![])
    }

    pub fn not_found_from<T: Into<Vec<u8>>>(content: T) -> HttpResponse {
        HttpResponse::new(HttpStatus::NotFound, content.into())
    }

    pub fn err_from<T: Into<Vec<u8>>>(content: T) -> HttpResponse {
        HttpResponse::new(HttpStatus::InternalSystemError, content.into())
    }

    pub fn err() -> HttpResponse {
        HttpResponse::new(HttpStatus::InternalSystemError, vec![])
    }

    pub fn bad_from<T: Into<Vec<u8>>>(content: T) -> HttpResponse {
        HttpResponse::new(HttpStatus::BadRequest, content.into())
    }

    pub fn bad() -> HttpResponse {
        HttpResponse::new(HttpStatus::BadRequest, vec![])
    }

    pub fn new_with_headers(status:HttpStatus, headers: HashMap<String, String>, content:impl Into<Vec<u8>>) -> HttpResponse {
        let mut res = HttpResponse { 
            status: status, 
            content: Box::new(content.into()),
            headers, 
        };
        res.headers.insert("Content-Lenght".to_string(), res.content_len().to_string());
        res
    }

    pub fn write(&self, mut stream: TcpStream) {
        stream.write_all(self.gen_first_line().as_bytes()).unwrap();
        stream.write_all(self.gen_headers().as_bytes()).unwrap();
        if self.content_len() > 0 {
            stream.write_all(&"\r\n".to_string().as_bytes());
            stream.write_all(&self.content); 
        }
    }

    fn gen_first_line(&self) -> String {
        format!("HTTP/1.1 {} {}\r\n", self.status.code(), self.status.name())
    }

    fn gen_headers(&self) -> String {
        let mut res = String::new();

        for header in &self.headers {
            res.push_str(&header.0);
            res.push_str(": ");
            res.push_str(&header.1);
            res.push_str("\r\n")
        }

        res
    }

    fn content_len(&self) -> usize {
        self.content.len()
    }

}

pub enum HttpStatus {
    Ok,
    NotFound,
    InternalSystemError,
    BadRequest,
}

impl HttpStatus {
    fn code(&self) -> &str {
        match self {
            HttpStatus::Ok => "200",
            HttpStatus::NotFound => "404",
            HttpStatus::InternalSystemError => "500",
            HttpStatus::BadRequest => "400",
        }
    }

    fn name(&self) -> &str {
        match self {
            HttpStatus::Ok => "OK",
            HttpStatus::NotFound => "NOT_FOUND",
            HttpStatus::InternalSystemError => "INTERNAL_SYSTEM_ERROR",
            HttpStatus::BadRequest => "BAD_REQUEST",
        }
    }

    #[deprecated(note = "replaced with code() and name()")]
    fn values(&self) -> (&str, &str) {
        match self {
            HttpStatus::Ok => ("200", "OK"),
            HttpStatus::NotFound => ("404", "NOT_FOUND"),
            HttpStatus::InternalSystemError => ("500", "INTERNAL_SYSTEM_ERROR"),
            HttpStatus::BadRequest => ("400", "BAD_REQUEST"),
        }
    }
}

