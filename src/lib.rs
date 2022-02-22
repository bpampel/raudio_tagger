use std::error::Error;
use std::fmt;
use std::io;
use std::io::Read;
use std::io::BufReader;
use std::fs::File;
use std::str;

mod id3;

#[derive(Debug)]
pub enum TagError {
    // specified container not found in file
    TagsNotFoundError,
    // something went wrong while parsing the tags
    ParseError,
    // Incorrect Header
    HeaderError,
    IoError(io::Error),
    Utf8Error(str::Utf8Error),
    FromUtf8Error(std::string::FromUtf8Error),
}

impl std::error::Error for TagError{}

impl fmt::Display for TagError{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "something went wrong… maybe be more explicit?")
    }
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let binary_data = read_bytewise_from_file(&config.filename)?;
    let mut f_tags = FileTags::new();

    match extract_id3(&binary_data, &mut f_tags) {
        Ok(_) => (),
        Err(TagError::ParseError) => panic!("Tag found but could not be parsed"),
        Err(e) => panic!("Some other error: {:?}",e),  // supress other warnings for the moment
    }
    f_tags.print_tags();
    Ok(())
}


pub fn read_bytewise_from_file(filename: &str) -> Result<Vec<u8>, io::Error> {
    let file = File::open(&filename)?;

    let mut reader = BufReader::new(file);
    let mut buffer = Vec::new();

    reader.read_to_end(&mut buffer)?;

    Ok(buffer)
}

pub fn extract_id3(data: &[u8], f_tags: &mut FileTags) -> Result<(), TagError> {
    let mut id3v1_tags = id3::ID3v1::create_from_binary(&data);
    let mut id3v2_tags = id3::ID3v2::create_from_binary(&data);
    match id3v1_tags {
        Ok(x) => {f_tags.id3v1 = Some(x);} ,
        Err(TagError::TagsNotFoundError) => (),  // ignore if not found
        Err(e) => return Err(e) //propagate any other error
    }
    match id3v2_tags {
        Ok(x) => {f_tags.id3v2 = Some(x);} ,
        Err(TagError::TagsNotFoundError) => (),  // ignore if not found
        Err(e) => return Err(e) //propagate any other error
    }
    Ok(())
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

// struct that collects all Tag variants for a File
pub struct FileTags {
    id3v1: Option<id3::ID3v1>,
    id3v2: Option<id3::ID3v2>,
}

impl FileTags {
    pub fn new() -> FileTags {
        FileTags{
            id3v1: None,
            id3v2: None,
        }
    }
    pub fn print_tags(self) {
        match self.id3v1 {
            Some(x) => println!("\nid3v1 found:\n{}", x),
            None => println!("\nid3v1 not found\n"),
        }
        match self.id3v2 {
            Some(x) => println!("\nid3v2 found:\n{}", x),
            None => println!("\nid3v2 not found\n"),
        }
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
    #[test]
    fn test_to_bitarray() {
        let byte = 5u8;
        let bitarr = BitArray::create_from_byte(byte, true);
        let expected = [false,false,false,false,false,true,false,true]; // 5 in bits, big endian
        assert_eq!(bitarr.bits, expected);
    }
}
