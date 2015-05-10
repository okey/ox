use std::fs::File;
use std::io;
use std::io::Cursor;
use std::io::prelude::*;
use byteorder::{BigEndian, ReadBytesExt};
//use disassemble::{DisassemblyResult, DisassemblyError};
//use disassemble::DisassemblyError::OpStreamError;
use disassemble::DisassemblyError;
use disassemble::DisassemblyError::DataError;

/*pub fn read_as_bytes(path: &String) -> io::Result<Vec<u8>> {
  let mut file = try!(File::open(&path));
  let mut v = Vec::new();
  match file.read_to_end(&mut v) {
    Ok(_) => Ok(v),
    Err(e) => Err(e),
  }
}*/

pub fn read_as_string(path: &String) -> io::Result<String> {
  let mut file = try!(File::open(&path));
  let mut s = String::new();
  match file.read_to_string(&mut s) {
    Ok(_) => Ok(s),
    Err(e) => Err(e),
  }
}

pub fn bytes_to_uint(data: &[u8]) -> Result<u32, DisassemblyError> {
  match data.len() {
    1 => Ok(data[0] as u32),
    2 => Ok(Cursor::new(data).read_u16::<BigEndian>().unwrap() as u32),
    4 => Ok(Cursor::new(data).read_u32::<BigEndian>().unwrap()),
    _ => {
      let d = data.len();
      let p = if d == 1 { "" } else { "s" };
      data_err!("Unsigned integer size ({} byte{}) not supported", d, p)
    }
  }
}

pub fn bytes_to_int(data: &[u8]) -> Result<i32, DisassemblyError> {
  match data.len() {
    //1 => Ok(data[0] as i32),
    2 => Ok(Cursor::new(data).read_i16::<BigEndian>().unwrap() as i32),
    4 => Ok(Cursor::new(data).read_i32::<BigEndian>().unwrap()),
    _ => {
      let d = data.len();
      let p = if d == 1 { "" } else { "s" };
      data_err!("Signed integer size ({} byte{}) not supported", d, p)
    }
  }
}

pub fn bytes_to_float(data: &[u8]) -> Result<f32, DisassemblyError> {
  match data.len() {
    4 => Ok(Cursor::new(data).read_f32::<BigEndian>().unwrap()),
    _ => {
      let d = data.len();
      let p = if d == 1 { "" } else { "s" };
      data_err!("Float size ({} byte{}) not supported", d, p)
    }
  }
}
