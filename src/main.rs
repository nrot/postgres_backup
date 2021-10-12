use std::io::{Error, ErrorKind};
use std::result::Result;
use std::env;
mod wal;
mod api;

use clap::clap_app;

fn print_main_help(){
    println!(r#"
        postgres_backup [COMMAND] [argvs]
        List of commands:
            help - 
            wal -  
    "#);
}

fn main() -> Result<(), Error> {
    println!("Start backup");

    dbgs!("Debugging enabled");

    // let command = env::args().nth(1);
    // dbgs!("Command {cmd}", cmd=command.clone().unwrap().as_str());

    let mathces = clap_app!(postgres_backup=>
        (version: "1.0")
        (author: "nrot <nrot13@gmail.com>")
        (@subcommand wal => 
            (@arg source: +required --source [FILE] "Путь до оригинального файла")
            (@arg filename: --filename [NAME] "Имя оригинального файла")
            (@arg dst_dir: --dst +required [PATH] "Путь до папки куда сохранять файл")
        )
        //(@arg COMMAND: +required "Команда")
        (@arg ehost: +required --elk [HOST] "host:port Хост и порт до logstash tcp сервера")
        (@arg password: +required --password [PASSWORD] "Пароль для отправки логов")
        (@arg index_name: --index [NAME] "Имя индекса для ELK")
        (@arg shost: --host [NAME] "Имя хоста от куда придет сообщение")
        (@arg zip: --zip "Сжимать ли бэкап. По умолчанию false")
    )
    .get_matches();

    match mathces.subcommand_name(){
        Some(s)=>{
            match s.to_lowercase().trim(){
                "wal"=>{
                    return wal::wal::wal_copy(mathces);
                },
                _=>{
                    print_main_help();
                    return Err(Error::new(ErrorKind::InvalidData, "Use help"));
                }
            }
        },
        _=>{
            print_main_help();
            return Err(Error::new(ErrorKind::NotFound, "Must be arvgs"));
        }
    }
}
