pub mod wal {
    use crate::api;
    use chrono::prelude;
    use clap::ArgMatches;
    use serde::{Deserialize, Serialize};
    use serde_json;
    use std::fs::{self, File};
    use std::io::prelude::Read;
    use std::io::{Error, ErrorKind, Write};
    use std::path::Path;
    use std::result::Result;

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

    pub fn wal_copy(argv: ArgMatches) -> Result<(), Error> {
        let timestamp = prelude::Local::now();
        crate::dbgs!("Run wal copy");

        let host = String::from(argv.value_of("ehost").expect("Host is required"));
        let password = String::from(argv.value_of("password").expect("Password is required"));
        let src = String::from(argv.value_of("source").expect("Source us required"));
        let filename = String::from(argv.value_of("filename").expect("Filename is required"));
        let dst = String::from(argv.value_of("dst_dir").expect("Dst dir is required"));
        let indx = String::from(argv.value_of("index_name").unwrap_or(&""));
        let shost = String::from(argv.value_of("shost").unwrap_or(&""));
        let zip = argv.is_present("zip");
        crate::dbgs!("Zip compress: {zip}", zip = argv.is_present("zip"));

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

        let dst_path = Path::new(&dst);
        crate::dbgs!("Source file: {s}", s = src.clone());
        crate::dbgs!(
            "Destination folder: {s}",
            s = dst_path.to_str().unwrap().clone()
        );

        match fs::canonicalize(&src) {
            Ok(src_path)=>{
                if !src_path.exists() || src_path.is_dir() {
                    msg.error = format!("Source file not exists or dir: {p}", p = src);
                } else if !(dst_path.exists() && dst_path.is_dir()) {
                    msg.error = format!("Destination dir does not exists: {path}", path = dst);
                } else {
                    let dst_file = dst_path.join(Path::new(&filename));
                    crate::dbgs!(
                        "Destination file: {s}",
                        s = dst_file.to_str().unwrap().clone()
                    );
                    if dst_file.exists() {
                        msg.error = format!(
                            "Can`t rewrite path: {path}",
                            path = dst_file.to_str().expect("Not empty filename")
                        );
                    } else {
                        if zip {
                            match File::open(src_path) {
                                Ok(mut src_f) => match File::create(dst_file.clone()) {
                                    Ok(back) => {
                                        let mut zip = zip::ZipWriter::new(&back);
                                        let options = zip::write::FileOptions::default()
                                            .compression_method(zip::CompressionMethod::Stored);
                                        match zip.start_file(filename.as_str(), options) {
                                            Ok(_) => {
                                                let mut buff = Vec::new();
                                                src_f.read_to_end(&mut buff)?;
                                                let writed = zip.write(&buff);
                                                match writed {
                                                    Ok(size) => {
                                                        msg.orig_size = size as u64;
                                                    }
                                                    Err(e) => {
                                                        msg.error =
                                                            format!("Can`t write ZIP file: {e}", e = e);
                                                    }
                                                }
                                                zip.finish().expect("Can`t finish file write");
                                            }
                                            Err(e) => {
                                                msg.error = format!("Can`t create ZIP file: {e}", e = e);
                                            }
                                        }
                                    }
                                    Err(e) => msg.error = format!("Can`t write backup file: {e}", e = e),
                                },
                                Err(e) => msg.error = format!("Can`t read backup file: {e}", e = e),
                            }
                        } else {
                            match std::fs::copy(src_path, dst_file.to_str().expect("Not empty filename")) {
                                Ok(size) => {
                                    msg.orig_size = size as u64;
                                }
                                Err(e) => {
                                    msg.error = format!("Can`t copy file: {e}", e = e);
                                }
                            }
                        }
                        msg.back_size = File::open(dst_file).expect("File can`t open").metadata()?.len() as u64;
                    }
                }
            },
            Err(e)=>{
                msg.error = format!("File does not exist: {e}", e=e);
            }
        }
        

        let error = msg.error.clone();
        msg.time_spent = (prelude::Local::now() - timestamp).num_milliseconds() as f64 / 1000.0;
        record.message = Some(msg);
        let data = serde_json::to_string(&record).unwrap();
        match api::send_tcp(&host, &data) {
            Ok(size) => {
                crate::dbgs!("Sended bytes: {s}", s = size);
                if !error.is_empty() {
                    crate::dbgs!("Error msg: {e}", e = error.clone());
                    return Err(Error::new(ErrorKind::Other, error));
                }
            }
            Err(e) => {
                println!("Error by sending to ELK {e}", e = e);
                return Err(e);
            }
        }

        return Ok(());
    }
}
