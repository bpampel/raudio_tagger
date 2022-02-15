use std::error::Error;
use std::io;
use std::io::Read;
use std::io::BufReader;
use std::fs::File;
use std::str;

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
}

pub fn run(config: &Config) -> Result<(), Box<dyn Error>> {
    let binary_data = read_bytewise_from_file(&config.filename)?;
    let mut f_tags = FileTags::new();

    match extract_id3(&binary_data, &mut f_tags) {
        Ok(_) => (),
        Err(TagError::ParseError) => panic!("Tag found but could not be parsed"),
        Err(_) => (),  // supress other warnings for the moment
    }
    //let stripped_data = extract_id3(&binary_data, &f_tags)?;
    //println!("{:?}", &binary_data[binary_data.len()-128..]);
    //println!("{:?}", &stripped_data);

    //let start_of_tag = binary_data.len() - ID3v1::LEN_BYTES;
    //println!("{:?}", &binary_data[start_of_tag..]);

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
    let mut id3v1_tags = ID3v1::create_from_binary(&data);
    match id3v1_tags {
        Ok(x) => {f_tags.id3v1 = Some(x);} ,
        Err(TagError::TagsNotFoundError) => (),  // ignore if not found
        Err(e) => return Err(e) //propagate any other error
    }
    Ok(())
    //f_tags.id3v1 = Some(
        //b"ID3" => {
            //let id3_len = (data[6] as usize) * 128 * 128 * 128
                //+ (data[7] as usize) * 128 * 128
                //+ (data[8] as usize) * 128
                //+ (data[9] as usize);

            //return Ok(&data[10 + id3_len..]);
        //}
    //}
}


//pub struct Format {
    //fields: Vec<String>,
//}


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
    id3v1: Option<ID3v1>,
    //id3v2: Optional<ID3v2>,
}

impl FileTags {
    pub fn new() -> FileTags {
        FileTags{ id3v1: None }
    }
}

pub struct ID3v1 {
    title: String,
    artist: String,
    album: String,
    year: u32,
    comment: String,
    track: Option<u8>,
    genre: u8,
}

impl ID3v1 {
    pub const LEN_BYTES: usize = 128;

    pub fn create_from_binary(file_data: &[u8]) -> Result<ID3v1, TagError> {
        // the tag is in the last 128 bytes starting with the string 'TAG'
        // 0..2 == 'TAG' (3 Bytes)
        // structure:
        // 3..32 == Song Name (30 bytes)
        // 33..62 == Artist (30 Bytes)
        // 63..92 == Album Name (30 Bytes)
        // 93..96 == Year (4 Bytes)
        // 97..124 or ..126 == Comment (28 or 30 Bytes)
        // 125 == Zero Byte: if this is 0, 126 might contain track number
        // 126 == Track Number if =! 0 and previous byte = 0
        // 127 == Song Genre Identifier (integer matching list)

        let start_of_tag = file_data.len() - ID3v1::LEN_BYTES;

        match &file_data[start_of_tag..start_of_tag + 3] {
            b"TAG" => {  // tag was found
                // slice the relevant part
                let id3_data = &file_data[start_of_tag..];
                // extract: I think the conversions should be always valid
                let title = str::from_utf8(&id3_data[3..32]).unwrap().to_string();
                let artist = str::from_utf8(&id3_data[33..62]).unwrap().to_string();
                let album = str::from_utf8(&id3_data[63..92]).unwrap().to_string();
                let year = u32::from_ne_bytes(id3_data[93..96].try_into().unwrap());

                // logic for the optional track number depending on the zero byte
                let mut track: Option<u8> = None;
                let comment: String;
                match id3_data[125] {
                    0u8 => {  // byte is zero, check if year is set
                        match id3_data[126] {
                            0u8 => {  // no year --> comment
                                comment = str::from_utf8(&id3_data[97..126]).unwrap().to_string();
                            }
                            t => {  // year is set
                                track = Some(t);
                                comment = str::from_utf8(&id3_data[97..124]).unwrap().to_string();
                            }
                        }
                    }
                    _ => {  // byte is non-zero --> long comment
                            comment = str::from_utf8(&id3_data[97..126]).unwrap().to_string();
                    }
                }
                let genre = &id3_data[127];


                Ok(ID3v1 { title, artist, album, year, comment, track, genre: *genre})
            }
            _ => return Err(TagError::TagsNotFoundError)
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
}
