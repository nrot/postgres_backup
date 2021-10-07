use chrono::prelude;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::{Error, ErrorKind, Write};
use std::net::TcpStream;
use std::path::Path;
use std::result::Result;
use std::time::Duration;
use clap::clap_app;

fn to_nanos(s: f64) -> u32 {
    (1_000_000_000.0 * s).floor() as u32
}

#[derive(Serialize, Deserialize)]
struct Message {
    source: String,
    filename: String,
    dst: String,
    error: String,
    file_size: u64,
}

#[derive(Serialize, Deserialize)]
struct Record {
    timestamp: String,
    index_name: String,
    version: String,
    password: String,
    host: String,
    message: Option<Message>,
}

fn main() -> Result<(), Error> {
    println!("Start backup");
    let timestamp = prelude::Local::now();

    let mathces = clap_app!(postgres_backup=>
        (version: "1.0")
        (author: "nrot <nrot13@gmail.com>")
        (@arg ehost: +required --elk [HOST] "host:port Хост и порт до logstash tcp сервера")
        (@arg password: +required --password [PASSWORD] "Пароль для отправки логов")
        (@arg source: +required --source [FILE] "Путь до оригинального файла")
        (@arg filename: --filename +required [NAME] "Имя оригинального файла")
        (@arg dst_dir: --dst +required [PATH] "Путь до папки куда сохранять файл")
        (@arg index_name: --index [NAME] "Имя индекса для ELK")
        (@arg shost: --host [NAME] "Имя хоста от куда придет сообщение")
    ).get_matches();

    let host = String::from(mathces.value_of("ehost").expect("Host is required"));
    let password = String::from(mathces.value_of("password").expect("Password is required"));
    let src = String::from(mathces.value_of("source").expect("Source us required"));
    let filename = String::from(mathces.value_of("filename").expect("Filenamme is required"));
    let dst = String::from(mathces.value_of("dst_dir").expect("Dst dir is required"));
    let indx = String::from(mathces.value_of("index_name").unwrap_or(&""));
    let shost = String::from(mathces.value_of("shost").unwrap_or(&""));

    let mut msg = Message {
        source: src.clone(),
        filename: filename.clone(),
        dst: dst.clone(),
        error: String::new(),
        file_size: 0,
    };
    let mut record = Record {
        timestamp: timestamp.format("%Y-%m-%dT%H:%M:%S.%3fZ").to_string(),
        password: password,
        index_name: indx,
        version: String::from("1"),
        host: shost.clone(),
        message: None,
    };

    if host.is_empty() {
        println!("ELK host is needed host:port -> {host}", host = host);
        return Err(Error::new(ErrorKind::InvalidInput, "ELK host is needed"));
    }
    let connection = TcpStream::connect(host);
    match connection {
        Ok(mut stream) => {
            let res = stream.set_write_timeout(Some(Duration::new(0, to_nanos(0.25))));
            match res {
                Ok(_) => {}
                Err(e) => {
                    println!("Set timeout error: {e}", e = e);
                    return Err(e);
                }
            };
            let dst_path = Path::new(&dst);
            if !(dst_path.exists() && dst_path.is_dir()) {
                println!("Destination dir does not exists: {path}", path = dst);
                msg.error = format!("Destination dir does not exists: {path}", path = dst);
                record.message = Some(msg);
                let answ = stream.write(serde_json::to_string(&record).unwrap().as_bytes());
                match answ {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Can`t send error message");
                        return Err(e);
                    }
                }
                return Err(Error::new(ErrorKind::NotFound, dst.as_str()));
            };
            println!("Try to copy file");
            let dst_file = dst_path.join(Path::new(&filename));
            if dst_file.exists() {
                println!(
                    "Can`t rewrite path: {path}",
                    path = dst_file.to_str().expect("Not empty filename")
                );
                msg.error = format!(
                    "Can`t rewrite path: {path}",
                    path = dst_file.to_str().expect("Not empty filename")
                );
                record.message = Some(msg);
                let answ = stream.write(serde_json::to_string(&record).unwrap().as_bytes());
                match answ {
                    Ok(_) => {}
                    Err(e) => {
                        println!("Can`t send error message");
                        return Err(e);
                    }
                }
                return Err(Error::new(
                    ErrorKind::NotFound,
                    dst_file.to_str().expect("Not empty filename"),
                ));
            }
            let copy_res = std::fs::copy(src, dst_file.to_str().expect("Not empty filename"));
            match copy_res {
                Ok(s) => {
                    msg.file_size = s;
                    record.message = Some(msg);
                    let answ = stream.write(serde_json::to_string(&record).unwrap().as_bytes());
                    match answ {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Can`t send error message");
                            return Err(e);
                        }
                    }
                }
                Err(e) => {
                    println!(
                        "Can`t copy file from: {dst} to {path}; Error: {e}",
                        dst = dst,
                        path = dst_file.to_str().expect("Not empty filename"),
                        e = e
                    );
                    msg.error = format!(
                        "Can`t copy file from: {dst} to {path}; Error: {e}",
                        dst = dst,
                        path = dst_file.to_str().expect("Not empty filename"),
                        e = e
                    );
                    record.message = Some(msg);
                    let answ = stream.write(serde_json::to_string(&record).unwrap().as_bytes());
                    match answ {
                        Ok(_) => {}
                        Err(e) => {
                            println!("Can`t send error message");
                            return Err(e);
                        }
                    }
                    return Err(e);
                }
            }
        }
        Err(e) => {
            println!("Connection ELK error: {e}", e = e);
            return Err(e);
        }
    }
    return Ok(());
}
