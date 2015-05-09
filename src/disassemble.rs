use std;
use std::collections::HashMap;
//use std::io::prelude::*;
use std::fs::File;
use std::io;
use std::io::{Read, Write, BufWriter, BufRead};
use std::iter::repeat;
use std::string::String;

use super::Routine;
use opcodes::{Opcode, Operand, NWType, get_nwtypes, OpPayload};
use io_utils::{bytes_to_uint, bytes_to_int, bytes_to_float};


const HEADER_BYTES: usize = 8;

pub enum DisassemblyError {
  DataError(String),
  IOError(io::Error),
  OpStreamError(String, usize)
}

impl From<io::Error> for DisassemblyError {
  fn from(e: io::Error) -> Self {
    DisassemblyError::IOError(e)
  }
}

// NOTE constraints between types and opcodes not really enforced, let alone strongly
// TODO redesign to fix this and make opcodes contingent upon types or something

use self::DisassemblyError::OpStreamError;
pub type DisassemblyResult = Result<(), DisassemblyError>;

pub fn format_output<'a, T: Write>(wtr: &mut T,
                               payload: &'a OpPayload,
                               opcodes: &'a [Option<Opcode>],
                               routines: &HashMap<u16, Routine>,
                               nwtypes: &[Option<NWType>]
                               ) -> Result<(), DisassemblyError>
{
  // TODO this needs to be recalculated to account for type suffixes
  // TODO don't do this here
  let longest_code = opcodes.iter()
    .filter_map(|c| match *c { Some(ref c) => Some(c.fmt.len()), None => None })
    .max().unwrap();
  let pad_str = String::from_utf8(repeat(0x20)
                                  .take(longest_code)
                                  .collect::<Vec<u8>>()
                                  ).unwrap();
  let op = payload.code; // TODO rename code
  let pad = longest_code - op.fmt.len();

  match payload._type {
    Some(byte) => match nwtypes.get(byte as usize).and_then(|t| t.as_ref()) {
      Some(typename) => match typename.abbr {
        Some(abbr) => {
          match op.types {
            Some(ref types) if 2 > types.len() => {
              output!(wtr, "{}{}", op.fmt, &pad_str[0..pad]);
            },
            _ => { output!(wtr, "{}{}{}", op.fmt, abbr, &pad_str[0..(pad-abbr.len())]); }
          }
        },
        None => { output!(wtr, "{}{}{:#04X}", op.fmt, &pad_str[0..pad], byte); }
      },
      None => { op_err!(0, "Undocumented type {} for opcode {}", byte, op.fmt) },
    },
    None => { output!(wtr, "{}{}", op.fmt, &pad_str[0..pad]); } // T
  }

  let args = &payload.args;
  // Variable length argument types (String) are preceded by a size argument
  let mut prev_val = None;

  // TODO formatting is broken, fix it
  // don't add trailing whitespace if there are no args
  // first arg indent needs to take into account variant type formatting for opcodes
  for pair in args.iter() { // why are we enumerating?
    let sep = &pad_str[0..5];
    let arg = &pair.0;
    let bytes = &pair.1;
    match **arg {
      Operand::Routine(size) | Operand::Object(size) | Operand::Size(size) => {
        let num = try!(bytes_to_uint(bytes.as_slice()));
        match **arg { // wish we had fallthrough because nesting this sucks :S
          Operand::Size(..) => {
            prev_val = Some(num);
            if op.types.is_none() {
              output!(wtr, "{:#010X}", num) // T - TODO this is horrific, clean it up
            } else {
              output!(wtr, "{}{:#X}", sep, num)
            }
          },
          Operand::Routine(..) => { // TODO force size of routine to be consistent with cast
            let id = num as u16; // consider parsing as u16 but storing as u32 in rtn struct?
            if let Some(f) = routines.get(&id) {
              output!(wtr, "{}{}#{:#X}", sep, f.name, num)
            } else {
              output!(wtr, "{}???#{}", sep, num)
            }
          },
          _ => output!(wtr, "{}{:#X}", sep, num)
        };
      },
      Operand::Offset(size) | Operand::Integer(size) | Operand::ArgCount(size) => {
        let num = try!(bytes_to_int(bytes.as_slice()));

        match **arg { // wish we had fallthrough because nesting this sucks :S
          Operand::Offset(..) => output!(wtr, "{}@{}", sep, num),
          _ => output!(wtr, "{}{}", sep, num)
        };
      },
      Operand::Float(size) => {
        let num = try!(bytes_to_float(bytes.as_slice()));
        output!(wtr, "{}{}", sep, num);
      },
      Operand::String => {
        let str_len = match prev_val {
          Some(n) => n as usize,
          None => {
            op_err!(0, "String argument without preceding length argument!");
          }
        };

        let s = std::str::from_utf8(bytes.as_slice()).unwrap();
        output!(wtr, "{}\"{}\"", sep, s);

      }
    }
  }
  try!(wtr.write(b"\n"));

  Ok(())
}

// TODO <T: Read>
// TODO decompiling will need an actual struct with opcode and stack args, probably
pub fn disassemble_op<'a, S: Read, T: Write>(asm: &mut S,
                                             opcodes: &'a [Option<Opcode>],
                                             wtr: &mut T,
                                             routines: &HashMap<u16, Routine>,
                                             nwtypes: &[Option<NWType>],
                                             bytecount: usize
                                             ) -> Result<OpPayload<'a>, DisassemblyError> {

  // Calculating this inside a loop is very inefficient...
  let longest_code = opcodes.iter()
    .filter_map(|c| match *c { Some(ref c) => Some(c.fmt.len()), None => None })
    .max().unwrap();
  let pad_str = String::from_utf8(repeat(0x20)
                                  .take(longest_code)
                                  .collect::<Vec<u8>>()
                                  ).unwrap();

  // Get a command byte and interpret it
  let mut byte_buf = [0 as u8; 1];
  let read_byte = read_exact!(asm, &mut byte_buf, byte_buf.len(), bytecount);

  let op = match opcodes.get(byte_buf[0] as usize).and_then(|c| c.as_ref()) {
    Some(op) => op,
    None => {
      op_err!(bytecount + read_byte, "Unknown opcode {:#04X}", byte_buf[0])
    }
  };
  let mut payload = OpPayload{ start: bytecount + read_byte, code: op, _type: None, args: vec!() };
  //payload.code = op.code;

  // Get the type byte - type of bytes that may be popped off the stack
  // determines legal args, but isn't necessarily the type of them
  let stack_type = match op.types {
    Some(ref t) => {
      payload.start += read_exact!(asm, &mut byte_buf, byte_buf.len(), payload.start);
      if t.contains(&byte_buf[0]) {
        payload._type = Some(byte_buf[0]);
        byte_buf[0]
      } else {
        op_err!(payload.start,
                "Type {:#04X} not in list of legal types for opcode {}", byte_buf[0], op.fmt);
      }
    },
    None => 0x00 // Hack for T
  };
  /*
  // Output opcode and type byte
  let pad = longest_code - op.fmt.len();
  // TODO this needs to be recalculated to account for type suffixes
  match nwtypes.get(stack_type as usize).and_then(|t| t.as_ref()) {
    Some(t) => match t.abbr {
      Some(a) => {
        match op.types {
          Some(ref types) if 2 > types.len() => {
            output!(wtr, "{}{}", op.fmt, &pad_str[0..pad]);
          },
          _ => {
            output!(wtr, "{}{}{}", op.fmt, a, &pad_str[0..(pad-a.len())]);
          }
        }
      },
      None => {
        if op.types.is_none() {
          output!(wtr, "{}{}", op.fmt, &pad_str[0..pad]); // T
        } else {
          output!(wtr, "{}{}{:#04X}", op.fmt, &pad_str[0..pad], stack_type);
        }
      }
    },
    None => {
      let msg = format!("Undocumented type {} for opcode {}", stack_type, op.fmt);
      return Err(OpStreamError(msg.to_string(), 0))
    }
  }
  */
  //let empty: Vec<Operand> = vec!(); // hack for arg extraction within loop
  // Get the arg list given the type byte
  let args = match op.args {
    Some(ref c) => {
      match c.get(&stack_type) {
        Some(a) => a,
        None => return Ok(payload)
      }
    },
    None => return Ok(payload)
  };

  // Variable length argument types (String) are preceded by a size argument
  let mut prev_val = None;

  // TODO formatting is broken, fix it
  // don't add trailing whitespace if there are no args
  // first arg indent needs to take into account variant type formatting for opcodes
  for (_, arg) in args.iter().enumerate() {
    // TODO use enumerate to get previous value from payload for strings?
    let sep = &pad_str[0..5];
    //let sep = if 0 == n { "" } else { full_sep };
    match *arg {
      // Could change ADT to be Operand(INT|UINT|FLT|STR, size) with INT(Offset|Integer) etc?
      Operand::Routine(size) | Operand::Object(size) | Operand::Size(size) => {
        let mut arg_vec = vec![0 as u8; size];
        payload.start += read_exact!(asm, arg_vec.as_mut_slice(), size, payload.start);
        let num = try!(bytes_to_uint(arg_vec.as_slice()));
        payload.args.push((arg, arg_vec));
        // TODO need to verify sizes here for others if we are returning raw bytes

        match *arg { // wish we had fallthrough because nesting this sucks :S
          Operand::Size(..) => {
            prev_val = Some(num);
            /*if op.types.is_none() {
              output!(wtr, "{:#010X}", num)
              // T - TODO this is horrific, clean it up
            } else {
              output!(wtr, "{}{:#X}", sep, num)
            }*/
          },
          _ => ()
          /*Operand::Routine(..) => { // TODO force size of routine to be consistent with cast
            let id = num as u16; // consider parsing as u16 but storing as u32 in rtn struct?
            if let Some(f) = routines.get(&id) {
              output!(wtr, "{}{}#{:#X}", sep, f.name, num)
            } else {
              output!(wtr, "{}???#{}", sep, num)
            }
          },
          _ => output!(wtr, "{}{:#X}", sep, num)*/
        };
      },
      Operand::Offset(size) | Operand::Integer(size) | Operand::ArgCount(size) => {
        let mut arg_vec = vec![0 as u8; size];
        payload.start += read_exact!(asm, arg_vec.as_mut_slice(), size, payload.start);
        payload.args.push((arg, arg_vec));
        /*let num = try!(bytes_to_int(arg_vec.as_slice()));

        match *arg { // wish we had fallthrough because nesting this sucks :S
          Operand::Offset(..) => output!(wtr, "{}@{}", sep, num),
          _ => output!(wtr, "{}{}", sep, num)
        };*/
      },
      Operand::Float(size) => {
        let mut arg_vec = vec![0 as u8; size];
        payload.start += read_exact!(asm, arg_vec.as_mut_slice(), size, payload.start);
        payload.args.push((arg, arg_vec));
        /*let num = try!(bytes_to_float(arg_vec.as_slice()));
        output!(wtr, "{}{}", sep, num);*/
      },
      Operand::String => {
        let str_len = match prev_val {
          Some(n) => n as usize,
          None => {
            op_err!(0, "String argument without preceding length argument!");
          }
        };

        let mut arg_vec = vec![0 as u8; str_len];
        payload.start += read_exact!(asm, arg_vec.as_mut_slice(), str_len, payload.start);
        payload.args.push((arg, arg_vec));
        /*let s = String::from_utf8(arg_vec).unwrap();
        output!(wtr, "{}\"{}\"", sep, s);*/

      }
    }
  }
  //try!(wtr.write(b"\n"));

  Ok(payload)
}

// TODO custom error type
pub fn disassemble<S: BufRead>(asm: &mut S, opcodes: &[Option<Opcode>],
                   routines: &HashMap<u16, Routine>,
                   input_name: &String,
                   filename: Option<String>) -> Result<(), DisassemblyError> {

  let nwtypes = get_nwtypes();
  let mut wtr = BufWriter::new(match filename {
    Some(path) => box try!(File::create(path)) as Box<Write>,
    None => box std::io::stdout() as Box<Write>
  });

  // The first HEADER_BYTES bytes should be a header string
  let mut header = [0 as u8; HEADER_BYTES];
  let mut bytes_read = read_exact!(asm, &mut header, header.len(), 0);
  output!(wtr, ";;{}\n", std::str::from_utf8(&header).unwrap());
  /*
  let longest_code = opcodes.iter()
    .filter_map(|c| match *c { Some(ref c) => Some(c.fmt.len()), None => None })
    .max().unwrap();
  let pad_str = String::from_utf8(repeat(0x20)
                                  .take(longest_code)
                                  .collect::<Vec<u8>>()
                                  ).unwrap();
  */

  let t = try!(disassemble_op(asm, opcodes, &mut wtr, routines, &nwtypes, bytes_read));
  let asm_len = try!(bytes_to_uint(t.args[0].1.as_slice())) as usize;
  bytes_read = t.start;
  //return Ok(());
  // TODO need a way to assert T is present
  // TODO need a way to return the value of T <arg> and check it against the file size
  // ...if possible
  // if streaming, count bytes, check for EOF, then check count against <arg>?

/*
  // T's sole operand is the file size - not sure if it's really unsigned though
  match opcodes.get(asm[HEADER_BYTES] as usize).and_then(|c| c.as_ref()) {
    Some(op) if op.code == 0x42 => {
      let asm_size_u32 =
        Cursor::new(&asm[HEADER_BYTES+1..start_idx]).read_u32::<BigEndian>().unwrap();

      if (asm_size_u32 as usize) != asm.len() {
        println_err!("T {} does not match file size ({} bytes)", asm_size_u32, asm.len());
        return Err(OpStreamError(DecodeError{message: "size mismatch".to_string(), byte: 0}))
      } else {
        // really need to set up an output stream or something
        // TODO get T.fmt etc from opcodes
        println!("T{}{:#010X}", &pad_str[0..longest_code - 1], asm_size_u32);
      }
    },
    Some(op) => {
      println_err!("Unexpected opcode {:#04X} at byte {}, expected T (0x42)",
                   op.code, HEADER_BYTES);
      return Err(OpStreamError(DecodeError{message: "T byte not present".to_string(), byte: 0}))
    },
    None => {
      println_err!("Unknown opcode {:#04X} at byte {}, expected T (0x42)",
                   asm[HEADER_BYTES], HEADER_BYTES);
      return Err(OpStreamError(DecodeError{message: "T byte not present".to_string(), byte: 0}))
    }
  }
  */

  // TODO allow user to specify decgimal or hex output for integers
  // TODO allow user to specify tabs or spaces



  /* Start parsing the command stream */

  // TODO handle special cases like SAVE_STATE (and T)
  loop {
    let c = try!(disassemble_op(asm, opcodes, &mut wtr, routines, &nwtypes, bytes_read));
    bytes_read = c.start;// TODO rename start
    try!(format_output(&mut wtr, &c, opcodes, routines, &nwtypes));
    if bytes_read == asm_len {
      break;
    }
  }

  Ok(())
}
