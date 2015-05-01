#![feature(plugin)]
#![feature(trace_macros)]
#![plugin(peg_syntax_ext)]
#![feature(str_char)]
//#![feature(exit_status)]
#![feature(collections)] // peg warnings
#![plugin(docopt_macros)]
extern crate rustc_serialize;
extern crate docopt;
extern crate byteorder;


use std::collections::HashMap;
use std::iter::repeat;
use std::io::{Cursor,Error};
use std::io::prelude::*;
use std::string::String;
use byteorder::{BigEndian, ReadBytesExt};

mod macros;
mod opcodes;
mod io_utils;
//mod disassemble;

use opcodes::Operand;
use io_utils::{read_as_bytes, read_as_string, bytes_to_uint, bytes_to_int, bytes_to_float};

peg_file! nwscript("nwscript.rustpeg");
use nwscript::document;


#[derive(Debug)]
pub struct Constant {
  typename: String,
  name: String,
  value: String
}

#[derive(Debug)]
pub struct Routine {
  return_type: String,
  name: String,
  code: u16,
  args: Vec<Arg>
}

#[derive(Debug)]
pub struct Arg {
  typename: String,
  name: String,
  default_value: Option<String>
}
#[derive(Debug)]
pub enum Statement {
  Rtn(Routine),
  Const(Constant)
}


const HEADER_BYTES: usize = 8;


fn build_tables(list: Vec<Statement>) -> (HashMap<String, Constant>, HashMap<u16, Routine>) {
  let mut constants = HashMap::new();
  let mut commands = HashMap::new(); // 16-bit, not sure if int or uint

  for st in list {
    match st {
      Statement::Const(c) => {
        match constants.insert(c.name.clone(), c) {
          Some(c) => {
            let d = constants.get(&c.name).unwrap();
            println_err!("Error: Multiple declarations of variable {}", d.name);
            println_err!("     > {} {} = {};", d.typename.trim(), d.name, d.value);
            panic!("duplicate variable");
          },
          None => ()
        }
      },
      Statement::Rtn(c) => {
        // This does not handle duplicate names, which would matter for compiling
        match commands.insert(c.code, c) {
          Some(c) => {
            let d = commands.get(&c.code).unwrap();
            println_err!("Error: Multiple declarations of routine {}", d.name);
            println_err!("     > {} {}(...) = {};", d.return_type.trim(), d.name, d.code);
            panic!("duplicate routine");
          },
          None => ()
        }
      }
    }
  }

  return (constants, commands)
}

#[derive(Debug)]
struct DecodeError {
  message: String,
  line: usize,
}

enum DisassemblyError {
  IOError(Error),
  CommandStreamError(DecodeError)
}

type DisassemblyResult<T> = Result<T, DisassemblyError>;

// TODO custom error type
fn disassemble(asm: &[u8], opcodes: &[Option<opcodes::Opcode>],
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

docopt!(Args derive Debug, "
Usage: ox -d <input.nsc> -c <def.ldf> [--nwn] [-o <output.oxa>]
       ox -a <input.oxa> [-c <def.ldf> [--nwn]] [-o <output.nsc>]
       ox --help

Options:
  -a INPUT    Assemble input.oxa file.
  -d INPUT    Disassemble input.nsc file.

  -c, --define DFILE      Engine routine definition file.
  --nwn                   Expect NWN-style routine definitions.
  -o, --output OUTPUT     The file to write output to.
  -h, --help              Show this message.
");
// gold-plating: tabs/spaces, hex options, cyclic (-r?) option that is -d then -a or vice versa

// -c is poorly named, and -d and --define are easily confused. TODO fix this.

fn main() {
  let args:Args = Args::docopt()
    .decode()
    .unwrap_or_else(|e| e.exit());

  println!("{:?} {}", args, args.flag_help);
    
  if args.flag_a.len() > 0 {
    println!("Assembling not yet implemented!");
    return
  }
  if args.flag_d.len() > 0 {
    
    let def_path = &args.flag_define;

    // Read the definitions file
    let res = match read_as_string(def_path) {
      Err(e) => panic!("{}", e),
      Ok(s) => s
    };

    // Parse definitions with peg
    let doc = match document(res.as_ref()) {
      Err(e) => panic!("{}", e),
      Ok(d) => d
    };

    // Build tables
    let (constants, routines) = build_tables(doc);
    println!("Read {} constants and {} routines", constants.len(), routines.len());

    // Read the compiled file
    let asm_path = &args.flag_d; // TODO stream this instead

    let asm = match read_as_bytes(asm_path) {
      Err(e) => panic!("{}", e),
      Ok(b) => b
    };
    
    let opcodes = opcodes::get_opcodes();
    let res = disassemble(&asm, &opcodes, &routines, asm_path, None);
    
    return
  }
}

#[cfg(test)]
mod nwscript_tests {
  use nwscript;

  #[test]
  fn function() {
    assert!(nwscript::function("void foo() = 0;").is_ok());
    assert!(nwscript::function("int foo(string x = \"\") = 10;").is_ok());
    assert!(nwscript::function("int foo(string x = \"\") = 10;").is_ok());

    assert!(nwscript::function("void foo();").is_err());
  }

  #[test]
  fn line_comments() {
    assert!(nwscript::line_comment("//this is a comment\n").is_ok());
    assert!(nwscript::function("int foo(string x = \"\")//hi\n = 10;").is_ok());
  }
}
