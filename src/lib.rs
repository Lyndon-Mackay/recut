use io::Read;
use std::{fs, io};

#[derive(Clone)]
pub enum IoType {
    FromStdIn,
    FromFile(String),
}

pub fn cut(input: IoType) -> Result<(), io::ErrorKind> {
    let contents = match input {
        IoType::FromFile(filename) => match fs::read_to_string(filename) {
            Ok(s) => Ok(s),
            Err(e) => Err(e),
        },
        IoType::FromStdIn => {
            let mut io_string = String::new();
            match io::stdin().read_to_string(&mut io_string) {
                Ok(_) => Ok(io_string),
                Err(e) => match e.kind() {
                    io::ErrorKind::BrokenPipe => Ok(io_string),
                    _ => Err(e),
                },
            }
        }
    };

    let contents = match contents {
        Ok(s) => s,
        Err(e) => return Err(e.kind()),
    };

    println!("{}", contents);
    Ok(())
}
