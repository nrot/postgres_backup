use std::io::{Error, ErrorKind};
use std::result::Result;
mod api;
mod wal;
mod full;
mod message;

use clap::clap_app;

fn print_main_help() {
    println!(
        r#"
        postgres_backup [COMMAND] [argvs]
        List of commands:
            help - 
            wal -  
    "#
    );
}

fn main() -> Result<(), Error> {
    println!("Start backup");

    dbgs!("Debugging enabled");

    // let command = env::args().nth(1);
    // dbgs!("Command {cmd}", cmd=command.clone().unwrap().as_str());

    let mathces = clap_app!(postgres_backup=>
        (version: "1.0")
        (author: "nrot <nrot13@gmail.com>")
        (about: "")
        (@arg COMMAND: +required [NAME] "Имя команды. wal/full")
        (@arg source: --source [FILE] "Путь до оригинального файла/пути")
        (@arg filename: --filename [NAME] "Имя оригинального файла")
        (@arg dst_dir: --dst +required [PATH] "Путь до папки куда сохранять файл")
        (@arg ehost: +required --elk [HOST] "host:port Хост и порт до logstash tcp сервера")
        (@arg password: +required --password [PASSWORD] "Пароль для отправки логов")
        (@arg index_name: --index [NAME] "Имя индекса для ELK")
        (@arg shost: --host [NAME] "Имя хоста от куда придет сообщение")
        (@arg zip: --zip "Сжимать ли бэкап. По умолчанию false")
        (@arg dbname: -d --dbname "Только для full Строка подключения заключенная в \"")
    )
    .get_matches();

    match mathces.value_of("COMMAND"){
        Some(s) => match s.to_lowercase().trim() {
            "wal" => {
                return wal::wal::wal_copy(mathces);
            },
            "full"=>{
                return full::full::full_copy(mathces);
            },
            c => {
                dbgs!("Main command: {s}", s=c);
                print_main_help();
                return Err(Error::new(ErrorKind::InvalidData, "Use help"));
            }
        },
        None => {
            print_main_help();
            return Err(Error::new(ErrorKind::NotFound, "Must be arvgs"));
        }
    }
}
