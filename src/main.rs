use chrono::prelude;
use clap::{clap_app};
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::prelude::{Read};
use std::io::{Error, ErrorKind, Write};
use std::net::TcpStream;
use std::path::Path;
use std::result::Result;
use std::time::Duration;
use std::fs::File;

fn to_nanos(s: f64) -> u32 {
    (1_000_000_000.0 * s).floor() as u32
}

#[derive(Serialize, Deserialize)]
struct Message {
    source: String,
    filename: String,
    dst: String,
    error: String,
    orig_size: u64,
    back_size: u64,
    time_spent: f64,
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
        (@arg filename: --filename [NAME] "Имя оригинального файла")
        (@arg dst_dir: --dst +required [PATH] "Путь до папки куда сохранять файл")
        (@arg index_name: --index [NAME] "Имя индекса для ELK")
        (@arg shost: --host [NAME] "Имя хоста от куда придет сообщение")
        (@arg zip: --zip "Сжимать ли бэкап. По умолчанию false")
    )
    .get_matches();

    let host = String::from(mathces.value_of("ehost").expect("Host is required"));
    let password = String::from(mathces.value_of("password").expect("Password is required"));
    let src = String::from(mathces.value_of("source").expect("Source us required"));
    let filename = String::from(mathces.value_of("filename").expect("Filenamme is required"));
    let dst = String::from(mathces.value_of("dst_dir").expect("Dst dir is required"));
    let indx = String::from(mathces.value_of("index_name").unwrap_or(&""));
    let shost = String::from(mathces.value_of("shost").unwrap_or(&""));
    let zip = mathces.is_present("zip");
    println!("Zip compress: {zip}", zip=mathces.is_present("zip"));

    let mut msg = Message {
        source: src.clone(),
        filename: filename.clone(),
        dst: dst.clone(),
        error: String::new(),
        orig_size: 0,
        back_size: 0,
        time_spent: 0.0,
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
                msg.time_spent =
                    (prelude::Local::now() - timestamp).num_milliseconds() as f64 / 1000.0;
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
                msg.time_spent =
                    (prelude::Local::now() - timestamp).num_milliseconds() as f64 / 1000.0;
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
            let copy_res: Result<u64, Error>;
            if zip {
                let src_f = File::open(src.clone());
                match src_f {
                    Ok(mut s) => {
                        let back = File::create(dst_file.clone());
                        match back {
                            Ok(b) => {
                                let mut zip = zip::ZipWriter::new(&b);
                                let options = zip::write::FileOptions::default().compression_method(zip::CompressionMethod::Stored);
                                zip.start_file(filename.as_str(), options).expect("Can`t append file to zip");
                                let mut buff = Vec::new();
                                s.read_to_end(&mut buff)?;
                                let writed = zip.write(&buff);
                                match writed{
                                    Ok(size)=>{
                                        copy_res = Ok(size as u64);
                                    },
                                    Err(e)=>{
                                        copy_res = Err(e);
                                    }
                                }
                                zip.finish().expect("Can`t finish file write");
                            }
                            Err(e) => {
                                msg.error = e.to_string();
                                msg.time_spent =
                                    (prelude::Local::now() - timestamp).num_milliseconds() as f64
                                        / 1000.0;
                                record.message = Some(msg);
                                let answ = stream
                                    .write(serde_json::to_string(&record).unwrap().as_bytes());
                                match answ {
                                    Ok(_) => {
                                        return Err(Error::new(
                                            ErrorKind::PermissionDenied,
                                            format!("Can`t read file: {dst}", dst = src.as_str()),
                                        ));
                                    }
                                    Err(e) => {
                                        println!("Can`t send error message");
                                        return Err(e);
                                    }
                                }
                            }
                        }
                    }
                    Err(e) => {
                        msg.error = e.to_string();
                        msg.time_spent =
                            (prelude::Local::now() - timestamp).num_milliseconds() as f64 / 1000.0;
                        record.message = Some(msg);
                        let answ = stream.write(serde_json::to_string(&record).unwrap().as_bytes());
                        match answ {
                            Ok(_) => {
                                return Err(Error::new(
                                    ErrorKind::PermissionDenied,
                                    format!("Can`t read file: {dst}", dst = src.as_str()),
                                ));
                            }
                            Err(e) => {
                                println!("Can`t send error message");
                                return Err(e);
                            }
                        }
                    }
                }
            } else {
                copy_res = std::fs::copy(src, dst_file.to_str().expect("Not empty filename"));
            }
            match copy_res {
                Ok(s) => {
                    msg.back_size = File::open(dst_file.clone())?.metadata()?.len() as u64;
                    msg.orig_size = s;
                    msg.time_spent =
                        (prelude::Local::now() - timestamp).num_milliseconds() as f64 / 1000.0;
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
                    msg.time_spent =
                        (prelude::Local::now() - timestamp).num_milliseconds() as f64 / 1000.0;
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
