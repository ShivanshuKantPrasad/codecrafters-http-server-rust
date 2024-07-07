use std::{
    io::{BufRead, BufReader, Write},
    net::TcpListener,
};

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                println!("accepted new connection");

                let buf_reader = BufReader::new(&mut stream);
                let http_request: Vec<_> = buf_reader
                    .lines()
                    .map(|result| result.unwrap())
                    .take_while(|line| !line.is_empty())
                    .collect();
                let request = http_request[0].clone();
                dbg!("{request}");
                let filepath = request.split_whitespace().nth(1).unwrap();
                dbg!("{filepath}");
                if filepath == "/" {
                    let _ = stream.write_all("HTTP/1.1 200 OK\r\n\r\n".as_bytes());
                } else {
                    let _ = stream.write_all("HTTP/1.1 404 Not Found\r\n\r\n".as_bytes());
                }
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
