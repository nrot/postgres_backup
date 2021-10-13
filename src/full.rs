pub mod full {
    use crate::{api, message};
    use chrono::prelude;
    use clap::ArgMatches;
    use serde_json;
    use std::io::{Error, ErrorKind};
    use std::process::Command;
    use std::result::Result;
    use std::str;

    pub fn full_copy(argv: ArgMatches) -> Result<(), Error> {
        let timestamp = prelude::Local::now();
        crate::dbgs!("Run full copy");

        let host = String::from(argv.value_of("ehost").expect("Host is required"));
        let password = String::from(argv.value_of("password").expect("Password is required"));
        let dst = String::from(argv.value_of("dst_dir").expect("Dst dir is required"));
        let indx = String::from(argv.value_of("index_name").unwrap_or(&""));
        let shost = String::from(argv.value_of("shost").unwrap_or(&""));
        let zip = argv.is_present("zip");
        let dbname = String::from(argv.value_of("dbname").unwrap_or(&""));
        crate::dbgs!("Zip compress: {zip}", zip = argv.is_present("zip"));

        let mut msg = message::Message {
            source: String::new(),
            filename: String::from("FULL COPY"),
            dst: dst.clone(),
            error: String::new(),
            orig_size: 0,
            back_size: 0,
            time_spent: 0.0,
        };
        let mut record = message::Record {
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

        match Command::new("pg_basebackup").arg("-v").status() {
            Ok(_) => {
                let mut pg_basebackup = Command::new("pg_basebackup");
                pg_basebackup.args(["-D", &dst]);
                pg_basebackup.arg("-Fp");
                pg_basebackup.arg("-R");
                if zip {
                    pg_basebackup.arg("-z");
                    pg_basebackup.args(["-Z", "9"]);
                }
                if cfg!(debug_assertions) {
                    pg_basebackup.arg("-P");
                    pg_basebackup.arg("--verbose");
                };
                if !dbname.is_empty() {
                    pg_basebackup.args(["--dbname", &dbname]);
                }
                match pg_basebackup.output() {
                    Ok(output) => {
                        println!(
                            "Success backup: {s}",
                            s = str::from_utf8(&output.stdout)
                                .expect("Can`t convert output ot string")
                        );
                    }
                    Err(e) => {
                        msg.error = format!("Execute error: {e}", e = e);
                    }
                }
            }
            Err(e) => {
                msg.error = format!("Command 'pg_basebackup -v' error: {e}", e = e);
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

        Ok(())
    }
}
