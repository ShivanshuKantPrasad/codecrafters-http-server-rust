use anyhow::Result;
use flate2::write::GzEncoder;
use flate2::Compression;
use std::collections::HashMap;
use std::fs;
use std::io::{BufRead, BufReader, Read};
use std::net::TcpStream;
use std::path::Path;
use std::{io::Write, net::TcpListener};

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
    method: HttpMethod,
    url: String,
    _version: HttpVersion,
    body: Option<String>,
    headers: HashMap<String, Vec<String>>,
}

impl HttpRequest {
    fn parse_request(stream: &mut TcpStream) -> Result<Self, String> {
        let mut buf_reader = BufReader::new(stream);
        let mut lines = buf_reader.by_ref().lines().map(|line| line.unwrap());

        let request_line = lines.next().unwrap();
        let request = request_line.split_whitespace().collect::<Vec<_>>();
        let method = match request.first() {
            Some(&"GET") => HttpMethod::Get,
            Some(&"POST") => HttpMethod::Post,
            _ => return Err(String::from("Http Header: Bad Method?")),
        };
        let url = match request.get(1) {
            Some(&url) => url.to_string(),
            None => return Err(String::from("Http Header: Missing URL?")),
        };
        let version = match request.last() {
            Some(&"HTTP/0.9") => HttpVersion::Http09,
            Some(&"HTTP/1.0") => HttpVersion::Http10,
            Some(&"HTTP/1.1") => HttpVersion::Http11,
            Some(&"HTTP/2.0") => HttpVersion::Http2,
            Some(&"HTTP/3.0") => HttpVersion::Http3,
            _ => return Err(String::from("Http Header: Bad Version?")),
        };

        let mut headers: HashMap<String, Vec<String>> = HashMap::default();
        for header in lines.by_ref() {
            if header.is_empty() {
                break;
            }
            match header.split_once(':') {
                Some((header, value)) => headers
                    .entry(header.to_string())
                    .or_default()
                    .push(value.trim().to_string()),
                None => return Err(String::from("Http header: Bad Header")),
            }
        }

        let body = match headers.get("Content-Length") {
            Some(_value) => {
                let length = _value[0].parse().unwrap();
                let mut body = vec![0; length];
                let _ = buf_reader.read_exact(&mut body);
                Some(String::from_utf8(body).unwrap())
            }
            None => None,
        };
        Ok(HttpRequest {
            method,
            url,
            _version: version,
            body,
            headers,
        })
    }
}

fn get(mut stream: TcpStream, http_request: HttpRequest) {
    // dbg!(&http_request);
    if http_request.url == "/" {
        // let _ = stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes());

        // Additional feature Serve a default index.html
        let args = std::env::args().collect::<Vec<_>>();
        let directory = args
            .iter()
            .skip_while(|x| **x != "--directory")
            .nth(1)
            .unwrap();
        let file_path = format!("{directory}index.html");
        let file_path = Path::new(file_path.as_str());
        if file_path.is_file() {
            let file_content = fs::read_to_string(file_path).unwrap();
            let _ = stream.write_all(format!("HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}", file_content.len(), file_content).as_bytes());
        }
        let _ = stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes());
    } else if http_request.url.starts_with("/echo") {
        let str = http_request.url.trim_start_matches("/echo/");
        let encoding = match http_request.headers.get("Accept-Encoding") {
            Some(x) => {
                if x[0].split(", ").any(|x| x == "gzip") {
                    "Content-Encoding: gzip\r\n"
                } else {
                    ""
                }
            }
            None => "",
        };
        let str = if !encoding.is_empty() {
            let mut e = GzEncoder::new(Vec::new(), Compression::default());
            let _ = e.write_all(str.as_bytes());
            e.finish().unwrap()
        } else {
            str.as_bytes().to_vec()
        };
        let buffer = [
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\n{}Content-Length: {}\r\n\r\n",
                encoding,
                str.len(),
            )
            .as_bytes(),
            str.as_slice(),
        ]
        .concat();
        let _ = stream.write_all(buffer.as_slice());
    } else if http_request.url.starts_with("/user-agent") {
        let str = http_request
            .headers
            .get("User-Agent")
            .unwrap()
            .first()
            .unwrap();
        let _ = stream.write_all(
            format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                str.len(),
                str
            )
            .as_bytes(),
        );
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

fn post(mut stream: TcpStream, http_request: HttpRequest) {
    if http_request.url.starts_with("/files") {
        let filename = http_request.url.strip_prefix("/files/").unwrap();
        let args = std::env::args().collect::<Vec<_>>();
        let directory = args
            .iter()
            .skip_while(|x| **x != "--directory")
            .nth(1)
            .unwrap();
        let _ = fs::write(format!("{directory}{filename}"), http_request.body.unwrap());
        let _ = stream.write_all("HTTP/1.1 201 Created\r\n\r\n".as_bytes());
    } else {
        let _ = stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes());
    }
}

fn handle_connection(mut stream: TcpStream) {
    println!("accepted new connection");

    match HttpRequest::parse_request(&mut stream) {
        Ok(http_request) => match http_request.method {
            HttpMethod::Post => post(stream, http_request),
            HttpMethod::Get => get(stream, http_request),
        },
        Err(err) => eprintln!("{err}"),
    }
}

fn main() -> Result<()> {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    println!("Listening at 127.0.0.1:4221");

    let args = std::env::args().collect::<Vec<_>>();
    let directory = args
        .iter()
        .skip_while(|x| **x != "--directory")
        .nth(1);

    if directory.is_some() { println!("Hosting the contents of {}", directory.unwrap()); }

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
    Ok(())
}
