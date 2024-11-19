#![allow(unused_imports)]
use std::collections::HashMap;
use std::str::from_utf8;
use std::time::{Duration, Instant};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

enum CMD {
    PING,
    ECHO(String),
    SET(String, String, Option<Instant>),
    GET(String),
}

impl CMD {
    pub fn response(self, env: &HashMap<String, (String, Option<Instant>)>) -> String {
        match self {
            CMD::PING => "+PONG\r\n".to_string(),
            CMD::ECHO(msg) => format!("+{}\r\n", msg),
            CMD::SET(..) => "+OK\r\n".to_string(),
            CMD::GET(key) => match env.get(&key) {
                Some((val, None)) => format!("${}\r\n{}\r\n", val.len(), val),
                Some((val, Some(expiry))) if Instant::now() < *expiry => {
                    format!("${}\r\n{}\r\n", val.len(), val)
                }
                _ => "$-1\r\n".to_string(),
            },
        }
    }

    pub fn run(&self, env: &mut HashMap<String, (String, Option<Instant>)>) {
        match self {
            CMD::SET(key, value, expiry) => {
                env.insert(key.clone(), (value.clone(), expiry.clone()));
            }
            _ => {}
        };
    }
}

#[tokio::main]
async fn main() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:6379").await.unwrap();

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
    let mut buf: [u8; 256] = [0; 256];
    let mut env: HashMap<String, (String, Option<Instant>)> = HashMap::new();

    loop {
        let bytes_read: usize = stream.read(&mut buf).await.unwrap();
        if bytes_read == 0 {
            break;
        }
        let cmd: CMD = resp_deserialize(from_utf8(&buf[..bytes_read]).unwrap());
        cmd.run(&mut env);
        stream.write(cmd.response(&env).as_bytes()).await.unwrap();
    }
}

fn resp_deserialize(resp: &str) -> CMD {
    let args: Vec<&str> = resp.split("\r\n").collect();
    match args[2] {
        "PING" => CMD::PING,
        "ECHO" => CMD::ECHO(args[4].to_string()),
        "SET" => CMD::SET(
            args[4].to_string(),
            args[6].to_string(),
            if args.len() > 8 && args[8].to_lowercase() == "px" {
                Some(Instant::now() + Duration::from_millis(args[10].parse::<u64>().unwrap()))
            } else {
                None
            },
        ),
        "GET" => CMD::GET(args[4].to_string()),
        _ => panic!("Unknown Command"),
    }
}
