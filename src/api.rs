use std::io::{Error, ErrorKind, Write};
use std::net::{TcpStream, Shutdown};
use std::result::Result;
use std::time::Duration;

pub fn send_tcp(host: &str, data: &str) -> Result<usize, Error>{
    let connection = TcpStream::connect(host);
    match connection{
        Ok(mut stream)=>{
            match stream.set_write_timeout(Some(Duration::new(0, to_nanos(0.25)))){
                Err(e)=>{
                    stream.shutdown(Shutdown::Both)?;
                    return Err(e)
                },
                Ok(_)=>{}
            }
            match stream.write(data.as_bytes()){
                Ok(size)=>{
                    stream.shutdown(Shutdown::Both)?;
                    return Ok(size);
                },
                Err(e)=>{
                    stream.shutdown(Shutdown::Both)?;
                    return Err(e);
                }
            }
        },
        Err(e)=>{
            return Err(e)
        }
    }
}

pub fn to_nanos(seconds: f64) -> u32 {
    (1_000_000_000.0 * seconds).floor() as u32
}

#[macro_export]
macro_rules! dbgs {
    ($($arg:tt)+) => {
        if cfg!(debug_assertions) {
            println!($($arg)+);
        };
    };
}