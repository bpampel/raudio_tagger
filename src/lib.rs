use std::io;
use std::io::Read;
use std::io::BufReader;
use std::fs::File;

pub fn read_bytewise_from_file(filename: &str) -> Result<Vec<u8>, io::Error> {
    let file = File::open(&filename)?;

    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    Ok(buffer)
}



pub struct Config {
    pub filename: String,
}

impl Config {
    pub fn new(args: &[String]) -> Result<Config, &str> {
        if args.len() < 2 {
            return Err("Not enough arguments specified");
        }
        let filename = args[1].clone();
        Ok(Config { filename })
    }
}



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_read() {
        let filename = "Test.mp3";
        let first_byte = 73u8;
        assert_eq!(first_byte, read_bytewise_from_file(filename).unwrap()[0]);
    }
}
