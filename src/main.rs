#![allow(unused_imports)]
use std::str::from_utf8;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

enum RESP {
    PING,
    ECHO(String),
}

#[tokio::main]
async fn main() {
    let listener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                println!("Connection Received");
                tokio::spawn(async move { handle_conn(stream).await });
            }
            Err(e) => println!("error: {}", e),
        }
    }
}

async fn handle_conn(mut stream: TcpStream) {
    let mut buf = [0; 256];

    loop {
        let n = stream.read(&mut buf).await.unwrap();
        if n == 0 {
            break;
        }
        stream
            .write(
                match resp_tokenize(from_utf8(&buf[0..n]).unwrap()) {
                    RESP::PING => "+PONG\r\n".to_string(),
                    RESP::ECHO(msg) => format!("+{}\r\n", msg),
                }
                .as_bytes(),
            )
            .await
            .unwrap();
    }
}

fn resp_tokenize(resp: &str) -> RESP {
    let args: Vec<&str> = resp.split("\r\n").collect();
    match args[2] {
        "PING" => RESP::PING,
        "ECHO" => RESP::ECHO(args[4].to_string()),
        _ => panic!("Unknown Command"),
    }
}
