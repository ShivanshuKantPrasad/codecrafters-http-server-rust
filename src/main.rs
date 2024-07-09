use std::collections::HashMap;
use std::fs;
use std::net::TcpStream;
use std::path::Path;
use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

#[derive(Debug)]
enum HttpMethod {
    Post,
    Get,
}

#[derive(Debug)]
enum HttpVersion {
    Http09,
    Http10,
    Http11,
    Http2,
    Http3,
}

#[derive(Debug)]
struct HttpRequest {
    _method: HttpMethod,
    url: String,
    _version: HttpVersion,
    _body: Option<String>,
    headers: HashMap<String, Vec<String>>,
}

impl HttpRequest {
    fn parse_request(http_request: Vec<String>) -> Result<Self, String> {
        let request_line = http_request[0].split_whitespace().collect::<Vec<_>>();
        let method = match request_line.first() {
            Some(&"GET") => HttpMethod::Get,
            Some(&"POST") => HttpMethod::Post,
            _ => return Err(String::from("Http Header: Bad Method?")),
        };
        let url = match request_line.get(1) {
            Some(&url) => url.to_string(),
            None => return Err(String::from("Http Header: Missing URL?")),
        };
        let version = match request_line.last() {
            Some(&"HTTP/0.9") => HttpVersion::Http09,
            Some(&"HTTP/1.0") => HttpVersion::Http10,
            Some(&"HTTP/1.1") => HttpVersion::Http11,
            Some(&"HTTP/2.0") => HttpVersion::Http2,
            Some(&"HTTP/3.0") => HttpVersion::Http3,
            _ => return Err(String::from("Http Header: Bad Version?")),
        };

        let mut headers: HashMap<String, Vec<String>> = HashMap::default();
        for header in http_request[1..].iter() {
            match header.split_once(':') {
                Some((header, value)) => headers
                    .entry(header.to_string())
                    .or_default()
                    .push(value.trim().to_string()),
                None => return Err(String::from("Http header: Bad Header")),
            }
        }
        Ok(HttpRequest {
            _method: method,
            url,
            _version: version,
            _body: None,
            headers,
        })
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("accepted new connection");

    let buf_reader = BufReader::new(&mut stream);
    let http_request: Vec<_> = buf_reader
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    match HttpRequest::parse_request(http_request) {
        Ok(http_request) => {
            // dbg!(&http_request);
            if http_request.url == "/" {
                let _ = stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes());
            } else if http_request.url.starts_with("/echo") {
                let str = http_request.url.trim_start_matches("/echo/");
                let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", str.len(), str).as_bytes());
            } else if http_request.url.starts_with("/user-agent") {
                let str = http_request
                    .headers
                    .get("User-Agent")
                    .unwrap()
                    .first()
                    .unwrap();
                let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}", str.len(), str).as_bytes());
            } else if http_request.url.starts_with("/files") {
                let filename = http_request.url.strip_prefix("/files/").unwrap();
                let args = std::env::args().collect::<Vec<_>>();
                let directory = args
                    .iter()
                    .skip_while(|x| **x != "--directory")
                    .nth(1)
                    .unwrap();
                dbg!(&directory);
                let file_path = format!("{directory}{filename}");
                let file_path = Path::new(file_path.as_str());
                if file_path.is_file() {
                    let file_content = fs::read_to_string(file_path).unwrap();
                    let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", file_content.len(), file_content).as_bytes());
                }
                let _ = stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes());
            } else {
                let _ = stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes());
            }
        }
        Err(err) => eprintln!("{err}"),
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                std::thread::spawn(|| handle_connection(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
