///! Module that contains the structure and parsing functions for id3 tags of mp3 files
///!
///! Both id3v1 and id3v2 are supported

use std::error;
use std::fmt;
use std::str;
use crate::TagError;
use encoding_rs;

pub struct ID3v1 {
    title: String,
    artist: String,
    album: String,
    year: u64,
    comment: String,
    track: Option<u8>,
    genre: u8,
}

impl ID3v1 {
    // v1 tags have a fixed size
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
                let title = unsafe_u8_to_str(&id3_data[3..33]).to_string();
                let artist = unsafe_u8_to_str(&id3_data[33..63]).to_string();
                let album = unsafe_u8_to_str(&id3_data[63..93]).to_string();

                // year is stored as string, transfer to int
                let year_str = unsafe_u8_to_str(&id3_data[93..97]);
                let year = match str::parse::<u64>(year_str) {
                    Ok(x) => x,
                    Err(_) => return Err(TagError::ParseError),
                };

                // logic for the optional track number depending on the zero byte
                let mut track: Option<u8> = None;
                let comment: String;
                match id3_data[125] {
                    0u8 => {  // byte is zero, check if year is set
                        match id3_data[126] {
                            0u8 => {  // no year --> comment
                                comment = unsafe_u8_to_str(&id3_data[97..127]).to_string();
                            }
                            t => {  // year is set
                                track = Some(t);
                                comment = unsafe_u8_to_str(&id3_data[97..125]).to_string();
                            }
                        }
                    }
                    _ => {  // byte is non-zero --> long comment
                            comment = unsafe_u8_to_str(&id3_data[97..127]).to_string();
                    }
                }
                let genre = &id3_data[127];


                Ok(ID3v1 { title, artist, album, year, comment, track, genre: *genre})
            }
            _ => return Err(TagError::TagsNotFoundError)
        }
    }
}

impl fmt::Display for ID3v1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.track {
            Some(t) => write!(f, "title: {}\nartist: {}\nalbum: {}\nyear: {}\ncomment: {}\ntrack: {}\ngenre: {}", self.title, self.artist,self.album,self.year,self.comment,t,self.genre),
            None => write!(f, "title: {}\nartist: {}\nalbum: {}\nyear: {}\ncomment: {}\ntrack: {}\ngenre:", self.title, self.artist,self.album,self.year,self.comment,self.genre),
        }
    }

}


pub struct ID3v2 {
    id3_version: u8,  // not really important currently
    id3_revision: u8,
    unsynchronization: bool,
    extended_header: bool,
    experimental_indicator: bool,
    size: usize,
    frames: Vec<ID3v2Frame>,
    //title: String,
    //artist: String,
    //album: String,
    //year: u64,
    //comment: String,
    //track: Option<u8>,
    //genre: u8,
}

impl ID3v2 {
    pub fn create_from_binary(file_data: &[u8]) -> Result<ID3v2, TagError> {
        // the tag structure is more complicated than in the v1 case
        // check if the file starts with ID3
        match &file_data[..3] {
            b"ID3" => {  // tag was found
                // slice the relevant part
                let header = &file_data[..10];
                let id3_version = &header[3];  // not really important currently
                let id3_revision = &header[4];
                let flags = BitArray::create_from_byte(header[5], true);
                let unsynchronization = flags.bits[0];
                let extended_header = flags.bits[1];
                let experimental_indicator = flags.bits[2];
                let size = ID3v2::calculate_size(&header[6..10]);
                let mut first_byte: usize = 10;
                let mut frames = Vec::new();
                while first_byte < size {
                    let frame = ID3v2::parse_frame(&file_data, first_byte, *id3_version);
                    match frame {
                        Ok(x) => {
                            frames.push(x.0);
                            first_byte = x.1 + 1;  // x.1 contains last byte of parsed frame
                        }
                        Err(e) => return Err(e),
                    }
                }

                Ok(ID3v2 { id3_version: *id3_version, id3_revision: *id3_revision, unsynchronization, extended_header, experimental_indicator, size, frames })
            }
            _ => return Err(TagError::TagsNotFoundError)
        }
    }
    fn calculate_size(bytes: &[u8]) -> usize {
        // without the first 10 bytes
        // encoded as 4 bytes with 7 bits:
        // cast to u32, use only last 7 bits and shift accordingly
        ((bytes[3] as u32 & 0x7F)
            + ((bytes[2] as u32 & 0x7F) << 7)
            + ((bytes[1] as u32 & 0x7F) << 14)
            + ((bytes[0] as u32 & 0x7F) << 21)
            ) as usize
    }
        fn parse_frame(file_data: &[u8], init: usize, version: u8) -> Result<(ID3v2Frame, usize), TagError> {
            let id: String;
            let size: u32;
            match version {
                2 => return Err(TagError::ParseError),  // currently not supported
                3 | 4 => {
                    let id = unsafe_u8_to_str(&file_data[init..init+4]).to_string();
                    println!("id: {}", id);
                    let size_vec = file_data[init+4..init+8].to_vec();
                    let size = read_be_u32(&mut &size_vec[..]) as usize;
                    let flags = [
                        BitArray::create_from_byte(file_data[init+9], true),
                        BitArray::create_from_byte(file_data[init+9], true),
                    ];
                    let frame = ID3v2Frame::create_from_bytes(&file_data[init..(init+10+size)], version, id, size, flags);
                    match frame {
                        Ok(x) => Ok((x, init+size)),
                        Err(e) => Err(e)
                    }
                }
            _ => return Err(TagError::ParseError)
            }
        }
    }

    #[derive(Default)]
    pub struct ID3v2Frame {
        version: u8,  // distinguish between v2 and later, as id changed size
        id: String,
        size: usize,
        flags: [BitArray; 2],
        data: String,
    }

impl ID3v2Frame {
    pub fn create_from_bytes(bytes: &[u8], version: u8, id: String, size: usize, flags: [BitArray;2]) -> Result<ID3v2Frame, TagError> {
        let data = match id.chars().next().unwrap() {
            'T' => { // text field
                let encoding = bytes[10];
                ID3v2Frame::decode_text_frame(&bytes[11..], encoding)?
            }
            x => {
                println!("{}",x);
                return Err(TagError::ParseError)
            }
        };
        //let data = match data_res {
            //Ok(x) => x,
            //Err(_) => return Err(TagError::ParseError),
        //}
        Ok(ID3v2Frame { version, id: id, size, flags, data} )
    }
    fn decode_text_frame(text_bytes: &[u8], encoding: u8) -> Result<String, TagError> {
        match encoding {
            0 => {  // ISO-8859-1
                return Err(TagError::ParseError)
            }
            1 => {  // UTF-16
                //let iter = (1..size)
                    //.map(|i| u16::from_be_bytes(&[bytes[2*i],&bytes[2*i+1]));
               ////for c in char::decode_utf16(bytes[1..]) {
                    //data = decode_utf16(iter).collect::<String>().ok();
                //let u16vec = BigEndian::read_u16(&bytes[1..]);
                //for elem in u16vec[..] {
                    //data.push(String::from_utf16(elem));
                return Err(TagError::ParseError)
            }
            2 => {  // UTF-16BE
                //let data = String::new();
                //let mut decoder = encoding_rs::UTF_16BE.new_decoder();
                //decoder.decode_to_string(&text_bytes[11..], &mut data, true);
                //println!("{}", data);
                return Err(TagError::ParseError)
            }
            3 => {  // UTF-8
                match String::from_utf8(text_bytes.to_vec()) {
                    Ok(x) => Ok(x),
                    Err(e) => return Err(TagError::FromUtf8Error(e)),
                }
            }
            _ => {
                println!("no encoding match");
                return Err(TagError::ParseError)
            }
        }
    }
    pub fn id_to_fieldname(id: &str) -> String {
        ///! lookup function that translates the frame ids to human readable strings
        String::new()
    }
}




impl fmt::Display for ID3v2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "id3_version: {}\nid3_revision: {}\nunsynchronization: {}\nextended_header: {}\nexperimental_indicator: {}\nsize: {}\n",self.id3_version, self.id3_revision,self.unsynchronization,self.extended_header,self.experimental_indicator,self.size)
    }

}


// helper structures and functions
#[derive(Default)]
pub struct BitArray {
    bits: [bool; 8],
    big_endian: bool,
}

impl BitArray {
    pub fn create_from_byte(byte: u8, big_endian: bool) -> BitArray {
        let mut tmp_byte = byte;
        let mut tmp_arr = [false; 8];  // initialize to 0
        for i in 0..8 {  // loop over bits
            let last_bit = (tmp_byte % 2) == 1;  // set to true or false
            match big_endian {
                true => { tmp_arr[7-i] = last_bit; },  // from back
                false => { tmp_arr[i] = last_bit; },
                }
            tmp_byte = tmp_byte >> 1;  // shift to next byte

        }
        BitArray { bits: tmp_arr, big_endian }

    }
}

fn unsafe_u8_to_str(u8data: &[u8]) -> &str {
    str::from_utf8(&u8data).unwrap()
}

// from rust docs
fn read_be_u32(input: &mut &[u8]) -> u32 {
    let (int_bytes, rest) = input.split_at(std::mem::size_of::<u32>());
    *input = rest;
    u32::from_be_bytes(int_bytes.try_into().unwrap())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_id3v2_size() {
        let bytes: [u8;4] = [1, 5, 7, 3]; // 0000001000010100001110000011
        let size = ID3v2::calculate_size(&bytes);
        let expected: u32 = 2097152 + 81920 + 896 + 3;
        assert_eq!(size, expected);
    }
}
