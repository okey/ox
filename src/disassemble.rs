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

// TODO what happens to control characters in strings? are they automatically escaped?

use self::DisassemblyError::OpStreamError;
pub type DisassemblyResult = Result<(), DisassemblyError>;

pub fn format_output<'a, T: Write>(wtr: &mut T,
                                   payload: &'a OpPayload,
                                   routines: &HashMap<u16, Routine>,
                                   nwtypes: &[Option<NWType>],
                                   pad_str: &String,
                                   ) -> Result<(), DisassemblyError>
{
  let op = payload.op;
  let longest_code = pad_str.len();
  let pad = longest_code - op.fmt.len();

  // print the opcode and flag whether or not the type byte was printed
  let mut skip_type = match payload._type {
    Some(byte) => match nwtypes.get(byte as usize).and_then(|t| t.as_ref()) {
      Some(typename) => match typename.abbr {
        Some(abbr) => {
          let n_args = payload.args.len();
          match op.types {
            Some(ref types) if 2 > types.len() => {
              let pad = if n_args > 0 { pad } else { 0 };
              output!(wtr, "{}{}", op.fmt, &pad_str[0..pad]); false
            },
            _ => {
              let abbr_pad = if pad >= abbr.len() && n_args > 0 { pad - abbr.len() } else { 0 };
              output!(wtr, "{}{}{}", op.fmt, abbr, &pad_str[0..abbr_pad]); true }
          }
        },
        None => match op.types {
          Some(ref types) if 2 > types.len() => {
            let pad = if payload.args.len() > 0 { pad } else { 0 };
            output!(wtr, "{}{}", op.fmt, &pad_str[0..pad]);
            true
          },
          _ => { output!(wtr, "{}{}{:#04X}", op.fmt, &pad_str[0..pad], byte); false }
        }
      },
      None => { op_err!(0, "Undocumented type {} for opcode {}", byte, op.fmt); },
    },
    None => { output!(wtr, "{}{}", op.fmt, &pad_str[0..pad]); false } // T
  };

  for (n, pair) in payload.args.iter().enumerate() {
    let sep = if skip_type { "" } else { &pad_str[0..5] };
    let arg = &pair.0;
    let bytes = &pair.1;
    match **arg {
      Operand::Routine(..) | Operand::Object(..) | Operand::Size(..) => {
        let num = try!(bytes_to_uint(bytes.as_slice()));
        match **arg { // wish we had fallthrough because nesting this sucks :S
          Operand::Size(..) => {
            match payload.args.get(n + 1) {
              Some(p) => match *p.0 {
                Operand::String => continue, // don't print the size before a string
                _ => ()
              },
              _ => ()
            }

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
              output!(wtr, "{}???#{:#X}", sep, num)
            }
          },
          _ => output!(wtr, "{}{:#X}", sep, num)
        };
      },
      Operand::ArgCount(..) => {
        let num = try!(bytes_to_uint(bytes.as_slice()));
        output!(wtr, "{}{}", sep, num);
      },
      Operand::Offset(..) | Operand::Integer(..) => {
        let num = try!(bytes_to_int(bytes.as_slice()));
        match **arg { // wish we had fallthrough because nesting this sucks :S
          Operand::Offset(..) => output!(wtr, "{}@{}", sep, num),
          _ => output!(wtr, "{}{}", sep, num)
        };
      },
      Operand::Float(..) => {
        let num = try!(bytes_to_float(bytes.as_slice()));
        output!(wtr, "{}{}", sep, num);
      },
      Operand::String => {
        let s = std::str::from_utf8(bytes.as_slice()).unwrap();
        output!(wtr, "{}\"{}\"", sep, s);
      }
    }
    skip_type = false;
  }
  try!(wtr.write(b"\n"));

  Ok(())
}

// TODO decompiling will need an actual struct with opcode and stack args, probably
pub fn disassemble_op<'a, T: Read>(asm: &mut T,
                                   opcodes: &'a [Option<Opcode>],
                                   byte_count: usize
                                   ) -> Result<OpPayload<'a>, DisassemblyError> {
  // Get a command byte and interpret it
  let mut byte_buf = [0 as u8; 1];
  let bytes_read = read_exact!(asm, &mut byte_buf, byte_buf.len(), byte_count);

  let op = match opcodes.get(byte_buf[0] as usize).and_then(|c| c.as_ref()) {
    Some(op) => op,
    None => {
      op_err!(byte_count + bytes_read, "Unknown opcode {:#04X}", byte_buf[0])
    }
  };
  let mut payload = OpPayload{ bytes_read: bytes_read, op: op, _type: None, args: vec!() };

  // Get the type byte - type of bytes that may be popped off the stack
  // determines legal args, but isn't necessarily the type of them
  let stack_type = match op.types {
    Some(ref t) => {
      payload.bytes_read += read_exact!(asm, &mut byte_buf, byte_buf.len(),
                                        byte_count + payload.bytes_read);
      if t.contains(&byte_buf[0]) {
        payload._type = Some(byte_buf[0]);
        byte_buf[0]
      } else {
        op_err!(byte_count + payload.bytes_read,
                "Type {:#04X} not in list of legal types for opcode {}", byte_buf[0], op.fmt);
      }
    },
    None => 0x00 // Hack for T
  };

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

  // this could go in a function
  for (_, arg) in args.iter().enumerate() {
    // TODO use enumerate to get previous value from payload for strings?
    match *arg {
      // Could change ADT to be Operand(INT|UINT|FLT|STR, size) with INT(Offset|Integer) etc?
      Operand::Routine(size) | Operand::Object(size) | Operand::Size(size) => {
        let mut arg_vec = vec![0 as u8; size];
        payload.bytes_read += read_exact!(asm, arg_vec.as_mut_slice(), size,
                                          byte_count + payload.bytes_read);
        let num = try!(bytes_to_uint(arg_vec.as_slice()));
        payload.args.push((arg, arg_vec));
        // TODO need to verify sizes here for others if we are returning raw bytes

        // TODO avoid needing prev_val? (lookbehind with index from enumerate?)
        match *arg { // wish we had fallthrough because nesting this sucks :S
          Operand::Size(..) => { prev_val = Some(num); },
          _ => ()
        };
      },
      Operand::Offset(size) | Operand::Integer(size) | Operand::ArgCount(size) => {
        let mut arg_vec = vec![0 as u8; size];
        payload.bytes_read += read_exact!(asm, arg_vec.as_mut_slice(), size,
                                          byte_count + payload.bytes_read);
        payload.args.push((arg, arg_vec));
      },
      Operand::Float(size) => {
        let mut arg_vec = vec![0 as u8; size];
        payload.bytes_read += read_exact!(asm, arg_vec.as_mut_slice(), size,
                                          byte_count + payload.bytes_read);
        payload.args.push((arg, arg_vec));
      },
      Operand::String => {
        let str_len = match prev_val {
          Some(n) => n as usize,
          None => {
            op_err!(0, "String argument without preceding length argument!");
          }
        };

        let mut arg_vec = vec![0 as u8; str_len];
        payload.bytes_read += read_exact!(asm, arg_vec.as_mut_slice(), str_len,
                                          byte_count + payload.bytes_read);
        payload.args.push((arg, arg_vec));
      }
    }
  }

  Ok(payload)
}

pub fn disassemble<S: BufRead>(asm: &mut S,
                               opcodes: &[Option<Opcode>],
                               routines: &HashMap<u16, Routine>,
                               output_name: Option<String>
                               ) -> Result<(), DisassemblyError> {

  let nwtypes = get_nwtypes();
  let mut wtr = BufWriter::new(match output_name {
    Some(path) => box try!(File::create(path)) as Box<Write>,
    None => box std::io::stdout() as Box<Write>
  });

  // The first HEADER_BYTES bytes should be a header string
  let mut header = [0 as u8; HEADER_BYTES];
  let mut bytes_read = read_exact!(asm, &mut header, header.len(), 0);
  output!(wtr, ";;{}\n", std::str::from_utf8(&header).unwrap());

  // Maybe payload & opcode should be the same type???
  let t = try!(disassemble_op(asm, opcodes, bytes_read));
  let expected_len = try!(bytes_to_uint(t.args[0].1.as_slice())) as usize;
  bytes_read += t.bytes_read;

  match t.op.code {
    0x42 => (),
    _ => {
      op_err!(t.bytes_read, "Unexpected opcode {:#04X}, expected T (0x42)", t.op.code);
    }
  }

  // Find the longest combination of opcode + type abbr (using only types legal for each op)
  let longest_code = opcodes.iter()
    .filter_map(|c| match *c {
      Some(ref c) => Some(c.fmt.len() + match c.types {
        Some(ref t) => {
          t.iter()
            .filter_map(|t| nwtypes.get(*t as usize))
            .map(|nwt| match *nwt {
              Some(ref nwt) => match nwt.abbr {
                Some(abbr) => abbr.len(),
                None => 0
              },
              None => 0
            })
            .max().unwrap()
        },
        None => 0
      }),
      None => None
    })
    .max().unwrap();

  // Generate a padding string for formatting indentation after opcodes
  let pad_str = String::from_utf8(repeat(0x20)
                                  .take(longest_code)
                                  .collect::<Vec<u8>>()
                                  ).unwrap();
  try!(format_output(&mut wtr, &t, routines, &nwtypes, &pad_str));



  // TODO allow user to specify decgimal or hex output for integers
  // TODO allow user to specify tabs or spaces

  /* Start parsing the command stream */
  // TODO handle special cases for STORE_STATE and co. to ensure they are followed by a JMP
  // and a block of code (block = RTN bounded)
  loop {
    let c = try!(disassemble_op(asm, opcodes, bytes_read));
    bytes_read += c.bytes_read;// TODO rename start
    try!(format_output(&mut wtr, &c, routines, &nwtypes, &pad_str));
    if bytes_read == expected_len {
      break;
    } else if bytes_read > expected_len {
      op_err!(bytes_read,
              "T {:#010X} does not match file size (read {} bytes)", expected_len, bytes_read);
    }
  }

  Ok(())
}
