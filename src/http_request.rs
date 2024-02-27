use std::fs::File;
use std::path::PathBuf;
use std::{net::TcpStream, collections::HashMap, str::FromStr};
use std::io::{Read, BufReader};
use serde::Deserialize;
use crate::err::HttpError;

use super::http_param::HttpParamValue;


#[derive(Debug)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Option,
}


#[derive(Debug)]
pub struct HttpRequest {
    pub method: HttpMethod,
    pub headers: HashMap<String,String>,
    pub uri: String,
    pub params: HashMap<String, HttpParamValue>,
    pub body: String,
    pub files: Vec<PathBuf>,
}

impl HttpRequest {
    pub fn new() -> HttpRequest {
        HttpRequest {
            method: HttpMethod::Get,
            headers: HashMap::new(),
            uri: String::new(),
            params: HashMap::new(),
            body: String::new(),
            files: vec![],
        }
    }

    pub fn get_param<T: FromStr>(&self, param_name: &str) -> Result<T, HttpError> {
        let param = self.params.get(param_name).ok_or_else(|| 
            HttpError::from_str(&format!("no param with name {param_name}"))
        )?;
        let HttpParamValue::Singular(param) = param else {
            return Err(HttpError::from_str(&format!("param: {param_name}:{:#?} was not singular", param)))
        };
        return param.parse::<T>().map_err(|_| 
                HttpError::from_str(&format!("couldn't parse param: {param_name}:{:#?}", param))
        );
    }
    
    pub fn body_as_json<'a, T>(&self) -> T where T:Deserialize<'a>{
        // let body: String = self.body.iter().cloned().collect();
        // let result = serde_json::from_str(&body).unwrap();
        // result
        todo!()
    }

    pub fn parse(stream: &mut TcpStream) -> Result<HttpRequest, HttpError> {
        parse_http_request(stream)
    }
    
}

fn take_til(reader: &mut BufReader<&mut TcpStream>, term: &[u8]) -> Vec<u8> {
    let mut vec: Vec<u8> = vec![];
    let mut reader = reader.bytes();

    loop {
        let b = match reader.next() {
            Some(Ok(b)) => b,
            Some(Err(_)) => panic!(),
            None => return vec,
        };
        vec.push(b);
        if term[term.len()-1] == b && vec.len() >= term.len() {
            if vec[vec.len()-term.len()..] == *term {
                return vec;
            } 
        }
    }
}
 
pub fn parse_http_request(stream: &mut TcpStream) -> Result<HttpRequest, HttpError> {
    let mut buf_reader = BufReader::new(&mut *stream);
    let header_part = take_til(&mut buf_reader, "\r\n\r\n".as_bytes());


    let mut result = parse_heading(&header_part)?;
    if result.headers.get("Content-Length").is_some_and(|cl| cl.parse::<u32>().unwrap() != 0) {

        match result.headers.get("Content-Type") {
            Some(multipart) if multipart.to_lowercase().contains("multipart") => {
                match parse_multipart_boundary(&header_part) {
                    Ok(boundary) => {
                        let body_part = take_til(&mut buf_reader, format!("{boundary}--").as_bytes());
                        let distribution = multipart_distribution(&body_part, boundary.as_bytes());
                        let files: Vec<PathBuf> = parse_multipart_parts(&body_part, distribution);
                        result.files = files;
                    }
                    Err(e) => return Err(HttpError::from(e))
                };
            }
            _ => {
                result.body = String::from_utf8(buf_reader.buffer().to_vec()).unwrap();
                return Ok(result);
            }
        }
    }

    Ok(result)
}

pub fn parse_multipart_parts(buffer: &[u8], distribution: MultipartDistribution) -> Vec<PathBuf> {
    let mut res = vec![];
    for part_index in 0..distribution.boundary_indexes.len() {
        let part_start = distribution.boundary_indexes[part_index]+distribution.boundary_len;
        let part_end = distribution.boundary_indexes.get(part_index+1).map(|x|x.clone()).unwrap_or(buffer.len());

        let mut headers = HashMap::new();

        let mut fc = part_start + 2; // first char of line
        let mut d = 0; // distance from fc
        let mut curr_header = String::new();
        let content_start = loop {
            if buffer[fc+d] == '\r' as u8 || buffer[fc+d] == '\n' as u8 {
                if buffer[fc+d+2] == '\r' as u8 || buffer[fc+d+2] == '\n' as u8 { // found "\r\n\r\n" no more headers
                    break fc+d+4;
                } else {
                    fc = fc+d+2; // skip "\r\n"
                    d = 0;
                    let xh: Vec<String> = curr_header.split(": ").map(str::to_string).collect();
                    headers.insert(xh[0].clone(),xh[1].clone());
                }
            } else {
                curr_header.push(buffer[fc+d] as char);
                d += 1;
            }
        };

        let path = PathBuf::from(format!("imported_file_{part_index}"));
        let mut file = File::create(&path).unwrap();
        std::io::Write::write(&mut file, &buffer[content_start..part_end]).unwrap();
        res.push(path);
    }

    return res;
}

fn parse_heading(buffer: &[u8]) -> Result<HttpRequest,HttpError> {
    let mut result = HttpRequest::new();
    let mut headers = HashMap::new();
    let x = std::str::from_utf8(&buffer).unwrap();



    if x.trim().trim_matches(char::from(0)).is_empty() { // this is the '\0' (end of string char)
        return Err(HttpError::from_str("empty request"))
    } 

    let mut line_iter = x.split("\n");
    let first_line = line_iter.next().expect("no first line");

    let mut first_line_parts = first_line.split(' ');
    let method_name = first_line_parts.next().unwrap();
    result.method = match method_name.trim() {
        "GET" => HttpMethod::Get,
        "POST" => HttpMethod::Post,
        "PUT" => HttpMethod::Put,
        "PATCH" => HttpMethod::Patch,
        "OPTION" => HttpMethod::Option,
        _ => {
            return Err(HttpError::from_str(&format!("Wrong method {method_name}")))
        }
    };
    
    let uri_and_params = first_line_parts.next().unwrap();
    result.params = parse_complex_params(uri_and_params);
    result.uri = uri_and_params.split("?").next().unwrap().to_string();

    for header_line in &mut line_iter { // headers
        if header_line.trim().trim_matches(char::from(0)).is_empty() { break; }

        let (key,value) = header_line.split_once(": ").expect(&format!("couldn't parse header param: {header_line}"));
        headers.insert(key.trim().to_string(), value.trim().to_string());
    }

    result.headers = headers;

    return Ok(result);

}

pub fn parse_multipart_boundary(buffer: &[u8]) -> Result<String, HttpError> {
    let boundary_templ = "boundary=".as_bytes();

    let mut i = 0;
    let mut f = 0;
    let boundary_start_index = loop {
        if buffer[i+f] == boundary_templ[f] {
            if f == boundary_templ.len() - 1 {
                break i+f+1;
            }
            f += 1;
        } else {
            f = 0;
            i += 1;
        }
    };  

    let mut j = 0;
    let mut res: Vec<u8> = vec![];
    loop {
        res.push(buffer[boundary_start_index+j]);

        let next = buffer[boundary_start_index+j+1];
        if  next == ';' as u8 || next == '\r' as u8 || next == '\n' as u8 {
            break;
        } else {
            j += 1;
        }
    };

    String::from_utf8(res).map_err(HttpError::from)
}

pub struct MultipartDistribution {pub boundary_indexes: Vec<usize>, pub boundary_len: usize}
pub fn multipart_distribution(multipart_request: &[u8], boundary: &[u8]) -> MultipartDistribution {
    let find_subsequence = |haystack: &[u8], needle: &[u8]| haystack.windows(needle.len()).position(|window| window == needle);
    let mut start_indexes: Vec<usize> = vec![];

    let mut next_start = 0;
    loop {
        let Some(part_len) = find_subsequence(&multipart_request[next_start..], &boundary) else {
            start_indexes.pop();
            break;
        };
        // let boundary_end = boundary_start + boundary.len();

        start_indexes.push(part_len + next_start);
        next_start = part_len + next_start + boundary.len();
    };
     
    MultipartDistribution { boundary_indexes: start_indexes, boundary_len: boundary.len() }
}


pub fn parse_complex_params(uri: &str) -> HashMap<String,HttpParamValue> {
    let mut result: HashMap<String,HttpParamValue> = HashMap::new();
    for (key, value) in parse_params(uri) {
        if value.is_empty() { continue; }
        let n_value: HttpParamValue;
        if value.contains(",") {
            n_value = HttpParamValue::List(value.split(",").map(String::from).collect());
        } else {
            n_value = HttpParamValue::Singular(value);
        }

        result.insert(key, n_value);
    };

    result

}


pub fn parse_params(uri: &str) -> HashMap<String,String>{
    let uri_parts: Vec<&str> = uri.split('?').collect();
    if uri_parts.len() == 1 { return HashMap::new() }

    let params: Vec<&str> = uri_parts.last().unwrap().split('&').collect();
    let mut param_map = HashMap::new();
    for param in params {
        let key_n_value: Vec<&str> = param.split('=').collect::<>();
        if key_n_value.len() == 1 { param_map.insert("".to_string(), key_n_value.last().unwrap().to_string()); }
        else if key_n_value.len() == 2 { param_map.insert(key_n_value.first().unwrap().to_string() ,key_n_value.last().unwrap().to_string()); }
    }

    param_map
}

