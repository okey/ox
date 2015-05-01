use std::fs::File;
use std::io::{Cursor, Result};
use std::io::prelude::*;
use byteorder::{BigEndian, ReadBytesExt};


pub fn read_as_bytes(path: &String) -> Result<Vec<u8>> {
  let mut file = try!(File::open(&path));
  let mut v = Vec::new();
  match file.read_to_end(&mut v) {
    Ok(_) => Ok(v),
    Err(e) => Err(e),
  }
}

pub fn read_as_string(path: &String) -> Result<String> {
  let mut file = try!(File::open(&path));
  let mut s = String::new();
  match file.read_to_string(&mut s) {
    Ok(_) => Ok(s),
    Err(e) => Err(e),
  }
}

pub fn bytes_to_uint(data: &[u8]) -> u32 {
  match data.len() {
    2 => Cursor::new(data).read_u16::<BigEndian>().unwrap() as u32,
    4 => Cursor::new(data).read_u32::<BigEndian>().unwrap(),
    _ => {
      let d = data.len();
      let p = if d == 1 { "" } else { "s" };
      println_err!("Error: Unsigned integer size ({} byte{}) not supported", d, p);
      panic!("unsupported integer size, {} byte{}", d, p);
    }
  }
}

pub fn bytes_to_int(data: &[u8]) -> i32 {
  match data.len() {
    1 => data[0] as i32,
    2 => Cursor::new(data).read_i16::<BigEndian>().unwrap() as i32,
    4 => Cursor::new(data).read_i32::<BigEndian>().unwrap(),
    _ => {
      let d = data.len();
      let p = if d == 1 { "" } else { "s" };
      println_err!("Error: Signed integer size ({} byte{}) not supported", d, p);
      panic!("unsupported integer size, {} byte{}", d, p);
    }
  }
}

pub fn bytes_to_float(data: &[u8]) -> f32 {
  match data.len() {
    4 => Cursor::new(data).read_f32::<BigEndian>().unwrap(),
    _ => {
      println_err!("Error: Float size ({} bytes) not supported", data.len());
      panic!("unsupported float size, {} bytes", data.len());
    }
  }
}
