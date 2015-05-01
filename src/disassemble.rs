use std;
use std::collections::HashMap;
use std::io::prelude::*;
use std::io::{Cursor,Error};
use std::iter::repeat;
use std::string::String;

use byteorder::{BigEndian, ReadBytesExt};

use super::{Routine};
use opcodes::{Opcode,Operand};
use io_utils::{bytes_to_uint, bytes_to_int, bytes_to_float};


const HEADER_BYTES: usize = 8;


#[derive(Debug)]
pub struct DecodeError {
  message: String,
  line: usize,
}

pub enum DisassemblyError {
  IOError(Error),
  CommandStreamError(DecodeError)
}

pub type DisassemblyResult<T> = Result<T, DisassemblyError>;

// TODO custom error type
pub fn disassemble(asm: &[u8], opcodes: &[Option<Opcode>],
               routines: &HashMap<u16, Routine>,
               input_name: &String,
               filename: Option<String>) -> DisassemblyResult<bool> {

  let fake_err = Err(DisassemblyError::CommandStreamError(DecodeError{message:"foo".to_string(), line: 0}));
  // The first HEADER_BYTES bytes should be a header string
  if asm.len() < HEADER_BYTES {
    println_err!("{} missing NWScript header bytes", input_name);
    return fake_err
  }
  println!(";;{}", std::str::from_utf8(&asm[..HEADER_BYTES]).unwrap());

  // TODO implement try!() or something for all these returns
  // This whole section is (almost) the same as one in the loop
  // make typename byte an Option?

  // The next 5 bytes are the T opcode - TODO get operand size from T
  let start_idx = HEADER_BYTES + 5;
  if asm.len() < start_idx {
    println_err!("{} missing NWScript size bytes", input_name);
    return fake_err
  }

  let longest_code = opcodes.iter()
    .filter_map(|c| match *c { Some(ref c) => Some(c.fmt.len()), None => None })
    .max().unwrap();
  let pad_str = String::from_utf8(repeat(0x20)
                                  .take(longest_code)
                                  .collect::<Vec<u8>>()
                                  ).unwrap();

  // T's sole operand is the file size - not sure if it's really unsigned though
  match opcodes.get(asm[HEADER_BYTES] as usize).and_then(|c| c.as_ref()) {
    Some(op) if op.code == 0x42 => {
      let asm_size_u32 =
        Cursor::new(&asm[HEADER_BYTES+1..start_idx]).read_u32::<BigEndian>().unwrap();

      if (asm_size_u32 as usize) != asm.len() {
        println_err!("T {} does not match file size ({} bytes)", asm_size_u32, asm.len());
        return fake_err
      } else {
        // really need to set up an output stream or something
        // TODO get T.fmt etc from opcodes
        println!("T{}{:#010X}", &pad_str[0..longest_code - 1], asm_size_u32);
      }
    },
    Some(op) => {
      println_err!("Unexpected opcode {:#04X} at byte {}, expected T (0x42)",
                   op.code, HEADER_BYTES);
      return fake_err
    },
    None => {
      println_err!("Unknown opcode {:#04X} at byte {}, expected T (0x42)",
                   asm[HEADER_BYTES], HEADER_BYTES);
      return fake_err
    }
  }

  // TODO allow user to specify decgimal or hex output for integers
  // TODO allow user to specify tabs or spaces



  /* Start parsing the command stream */

  // TODO handle special cases like SAVE_STATE
  let asm_len = asm.len();
  let empty: Vec<Operand> = vec!(); // hack for arg extraction within loop
  let mut idx = (start_idx, start_idx);

  loop {
    idx = step_or_return!(idx, 1, asm_len, fake_err); // .0 => .1, .1 => .1 + step

    // Get a command byte and interpret it
    let op = match opcodes.get(asm[idx.0] as usize).and_then(|c| c.as_ref()) {
      Some(op) => op,
      None => {
        println_err!("Unknown opcode {:#04X} at byte {}", asm[idx.0], idx.0);
        return fake_err
      }
    };

    // Get the type byte - type of bytes that may be popped off the stack
    // determines legal args, but isn't necessarily the type of them
    idx = step_or_return!(idx, 1, asm_len, fake_err);
    // TODO make type an Option? To handle T etc
    let stack_type = if op.types.contains(&asm[idx.0]) {
      asm[idx.0]
    } else {
      println_err!("Type {:#04X} not in list of legal types for opcode {}",
                   asm[idx.0], op.fmt);
      return fake_err
    };

    let pad = longest_code - op.fmt.len();
    print!("{}{}{:#04X}", op.fmt, &pad_str[0..pad], stack_type);


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

    for arg in args {
      match *arg {
        // Could change ADT to be Operand(INT|UINT|FLT|STR, size) with INT(Offset|Integer) etc?
        Operand::Routine(size) | Operand::Object(size) | Operand::Size(size) => {
          idx = step_or_return!(idx, size, asm_len, fake_err);
          let num = bytes_to_uint(&asm[idx.0..idx.1]);

          match *arg { // wish we had fallthrough because nesting this sucks :S
            Operand::Size(..) => {
              prev_val = Some(num);
              print!("\t{:#X}", num)
            },
            Operand::Routine(..) => {
              let id = num as u16; // consider parsing as u16 but storing as u32 in rtn struct?
              if let Some(f) = routines.get(&id) {
                print!("\t{}#{:#X}", f.name, num)
              } else {
                print!("\t???#{}", num)
              }
            },
            _ => print!("\t{:#X}", num)
          };
        },
        Operand::Offset(size) | Operand::Integer(size) | Operand::ArgCount(size) => {
          idx = step_or_return!(idx, size, asm_len, fake_err);
          let num = bytes_to_int(&asm[idx.0..idx.1]);

          match *arg { // wish we had fallthrough because nesting this sucks :S
            Operand::Offset(..) => print!("\t@{}", num),
            _ => print!("\t{}", num)
          };
        },
        Operand::Float(size) => {
          idx = step_or_return!(idx, size, asm_len, fake_err);
          let num = bytes_to_float(&asm[idx.0..idx.1]);
          print!("\t{}", num);
        },
        Operand::String => {
          let str_len = match prev_val {
            Some(n) => n as usize,
            None => {
              println_err!("\nString argument without preceding length argument!");
              return fake_err
            }
          };

          idx = step_or_return!(idx, str_len, asm_len, fake_err);
          let s = std::str::from_utf8(&asm[idx.0..idx.1]).unwrap();
          print!("\t\"{}\"", s);

        }
        // TODO errors/streams etc
      }
    }
    println!("");
    if idx.0 == asm_len {
      break
    }
  }

  fake_err
}
