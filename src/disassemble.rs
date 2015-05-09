use std;
use std::collections::HashMap;
//use std::io::prelude::*;
use std::fs::File;
use std::io::{Error, Read, Write, BufWriter, BufRead};
use std::iter::repeat;
use std::string::String;

use super::Routine;
use opcodes::{Opcode, Operand, NWType, get_nwtypes};
use io_utils::{bytes_to_uint, bytes_to_int, bytes_to_float};


const HEADER_BYTES: usize = 8;


#[derive(Debug)]
pub struct DecodeError {
  pub message: String,
  byte: usize,
}

pub enum DisassemblyError {
  IOError(Error),
  CommandStreamError(DecodeError)
}

impl From<Error> for DisassemblyError {
  fn from(e: Error) -> Self {
    DisassemblyError::IOError(e)
  }
}

// NOTE constraints between types and opcodes not really enforced, let alone strongly
// TODO redesign to fix this and make opcodes contingent upon types or something

use self::DisassemblyError::CommandStreamError;
pub type DisassemblyResult = Result<(), DisassemblyError>;

// TODO <T: Read>
// TODO decompiling will need an actual struct with opcode and stack args, probably
pub fn disassemble_op<S: Read, T: Write>(asm: &mut S,
                                         opcodes: &[Option<Opcode>],
                                         output: &mut T,
                                         routines: &HashMap<u16, Routine>,
                                         nwtypes: &[Option<NWType>]
                                         ) -> Result<(()), DisassemblyError> {

  let count = 0; // TODO pass in?
  let empty: Vec<Operand> = vec!(); // hack for arg extraction within loop
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
  read_exact!(asm, &mut byte_buf, byte_buf.len());

  let op = match opcodes.get(byte_buf[0] as usize).and_then(|c| c.as_ref()) {
    Some(op) => op,
    None => {
      println_err!("Unknown opcode {:#04X} at byte {}", byte_buf[0], count);
      return Err(CommandStreamError(DecodeError{message: "unknown opcode".to_string(), byte: 0}))
    }
  };

  // Get the type byte - type of bytes that may be popped off the stack
  // determines legal args, but isn't necessarily the type of them
  let stack_type = match op.types {
    Some(ref t) => {
      read_exact!(asm, &mut byte_buf, byte_buf.len());
      if t.contains(&byte_buf[0]) {
        byte_buf[0]
      } else {
        println_err!("Type {:#04X} not in list of legal types for opcode {}", byte_buf[0], op.fmt);
        return Err(CommandStreamError(DecodeError{message: "illegal type".to_string(), byte: 0}))
      }
    },
    None => 0x00 // Hack for T
  };

  let pad = longest_code - op.fmt.len();
  // TODO this needs to be recalculated to account for type suffixes
  match nwtypes.get(stack_type as usize).and_then(|t| t.as_ref()) {
    Some(t) => match t.abbr {
      Some(a) => {
        match op.types {
          Some(ref types) if 2 > types.len() => {
            try!(output.write(format!("{}{}", op.fmt, &pad_str[0..pad]).as_bytes()));
          },
          _ => {
            try!(output.write(format!("{}{}{}",
                                      op.fmt, a, &pad_str[0..(pad-a.len())]).as_bytes()));
          }
        }
      },
      None => {
        if op.types.is_none() {
          try!(output.write(format!("{}{}", op.fmt, &pad_str[0..pad]).as_bytes())); // T
        } else {
          try!(output.write(format!("{}{}{:#04X}",
                                    op.fmt, &pad_str[0..pad], stack_type).as_bytes()));
        }
      }
    },
    None => {
      println_err!("Undocumented type {} for opcode {}", stack_type, op.fmt);
      return Err(CommandStreamError(DecodeError{message: "Undocumented type".to_string(), byte: 0}))
    }
  }

  // Get the arg list given the type byte
  let args = match op.args {
    Some(ref c) => {
      match c.get(&stack_type) {
        Some(a) => a,
        None => &empty
      }
    },
    None => &empty
  };

  // Variable length argument types (String) are preceded by a size argument
  let mut prev_val = None;

  // TODO formatting is broke, fix it
  // don't add trailing whitespace if there are no args
  // first arg indent needs to take into account variant type formatting for opcodes
  for (_, arg) in args.iter().enumerate() {
    let sep = &pad_str[0..5];
    //let sep = if 0 == n { "" } else { full_sep };
    match *arg {
      // Could change ADT to be Operand(INT|UINT|FLT|STR, size) with INT(Offset|Integer) etc?
      Operand::Routine(size) | Operand::Object(size) | Operand::Size(size) => {
        let mut arg_vec = vec![0 as u8; size];;
        read_exact!(asm, arg_vec.as_mut_slice(), size);
        let num = bytes_to_uint(arg_vec.as_slice());

        match *arg { // wish we had fallthrough because nesting this sucks :S
          Operand::Size(..) => {
            prev_val = Some(num);
            if op.types.is_none() {
              try!(output.write(format!("{:#010X}", num).as_bytes()))
              // T - TODO this is horrific, clean it up
            } else {
              try!(output.write(format!("{}{:#X}", sep, num).as_bytes()))
            }
          },
          Operand::Routine(..) => {
            let id = num as u16; // consider parsing as u16 but storing as u32 in rtn struct?
            if let Some(f) = routines.get(&id) {
              try!(output.write(format!("{}{}#{:#X}", sep, f.name, num).as_bytes()))
            } else {
              try!(output.write(format!("{}???#{}", sep, num).as_bytes()))
            }
          },
          _ => try!(output.write(format!("{}{:#X}", sep, num).as_bytes()))
        };
      },
      Operand::Offset(size) | Operand::Integer(size) | Operand::ArgCount(size) => {
        let mut arg_vec = vec![0 as u8; size];
        read_exact!(asm, arg_vec.as_mut_slice(), size);
        let num = bytes_to_int(arg_vec.as_slice());

        match *arg { // wish we had fallthrough because nesting this sucks :S
          Operand::Offset(..) => try!(output.write(format!("{}@{}", sep, num).as_bytes())),
          _ => try!(output.write(format!("{}{}", sep, num).as_bytes()))
        };
      },
      Operand::Float(size) => {
        let mut arg_vec = vec![0 as u8; size];
        read_exact!(asm, arg_vec.as_mut_slice(), size);
        let num = bytes_to_float(arg_vec.as_slice());
        try!(output.write(format!("{}{}", sep, num).as_bytes()));
      },
      Operand::String => {
        let str_len = match prev_val {
          Some(n) => n as usize,
          None => {
            println_err!("\nString argument without preceding length argument!");
            return Err(CommandStreamError(DecodeError{message: "String without size".to_string(), byte: 0}))
          }
        };

        let mut arg_vec = vec![0 as u8; str_len];
        read_exact!(asm, arg_vec.as_mut_slice(), str_len);
        let s = String::from_utf8(arg_vec).unwrap();
        try!(output.write(format!("{}\"{}\"", sep, s).as_bytes()));

      }
      // TODO errors/streams etc
    }
  }
  try!(output.write(b"\n"));

  Ok(())
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
  read_exact!(asm, &mut header, header.len());
  try!(wtr.write(format!(";;{}\n", std::str::from_utf8(&header).unwrap()).as_bytes()));
  /*
  let longest_code = opcodes.iter()
    .filter_map(|c| match *c { Some(ref c) => Some(c.fmt.len()), None => None })
    .max().unwrap();
  let pad_str = String::from_utf8(repeat(0x20)
                                  .take(longest_code)
                                  .collect::<Vec<u8>>()
                                  ).unwrap();
  */

  try!(disassemble_op(asm, opcodes, &mut wtr, routines, &nwtypes));
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
        return Err(CommandStreamError(DecodeError{message: "size mismatch".to_string(), byte: 0}))
      } else {
        // really need to set up an output stream or something
        // TODO get T.fmt etc from opcodes
        println!("T{}{:#010X}", &pad_str[0..longest_code - 1], asm_size_u32);
      }
    },
    Some(op) => {
      println_err!("Unexpected opcode {:#04X} at byte {}, expected T (0x42)",
                   op.code, HEADER_BYTES);
      return Err(CommandStreamError(DecodeError{message: "T byte not present".to_string(), byte: 0}))
    },
    None => {
      println_err!("Unknown opcode {:#04X} at byte {}, expected T (0x42)",
                   asm[HEADER_BYTES], HEADER_BYTES);
      return Err(CommandStreamError(DecodeError{message: "T byte not present".to_string(), byte: 0}))
    }
  }
  */

  // TODO allow user to specify decgimal or hex output for integers
  // TODO allow user to specify tabs or spaces



  /* Start parsing the command stream */

  // TODO handle special cases like SAVE_STATE (and T)
  loop {
    try!(disassemble_op(asm, opcodes, &mut wtr, routines, &nwtypes));
  }

  Ok(())
}
