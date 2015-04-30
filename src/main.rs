#![feature(plugin)]
#![feature(trace_macros)] 
#![plugin(peg_syntax_ext)]
#![feature(str_char)]

// peg warnings
#![feature(collections)]

extern crate byteorder;

use std::env;

use std::fs::File;

use std::io;
use std::io::prelude::*;

use std::collections::HashMap;

use std::string::String;

use std::io::Cursor;
use byteorder::{BigEndian, ReadBytesExt};

use std::iter::repeat;

peg_file! nwscript("nwscript.rustpeg");
use nwscript::document;

mod opcodes;
use opcodes::Operand;


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


macro_rules! step_or_fail {
  ($t:ident, $n:expr, $lim:expr) => (
    if ($t.1 + $n) < $lim { ($t.1, $t.1 + $n) } else { return; }
    )
}

macro_rules! println_err(
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
);


const HEADER_BYTES: usize = 8;


fn read_as_bytes(path: &String) -> io::Result<Vec<u8>> {
  let mut file = try!(File::open(&path));
  let mut v = Vec::new();
  match file.read_to_end(&mut v) {
    Ok(_) => Ok(v),
    Err(e) => Err(e),
  }
}

fn read_as_string(path: &String) -> io::Result<String> {
  let mut file = try!(File::open(&path));
  let mut s = String::new();
  match file.read_to_string(&mut s) {
    Ok(_) => Ok(s),
    Err(e) => Err(e),
  }
}

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

fn bytes_to_uint(data: &[u8]) -> u32 {
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

fn bytes_to_int(data: &[u8]) -> i32 {
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

fn bytes_to_float(data: &[u8]) -> f32 {
  match data.len() {
    4 => Cursor::new(data).read_f32::<BigEndian>().unwrap(),
    _ => {
      println_err!("Error: Float size ({} bytes) not supported", data.len());
      panic!("unsupported float size, {} bytes", data.len());
    }
  }
}

fn main() {
  let args:Vec<_> = env::args().collect();

  if args.len() == 3 {
    let def_path = &args[1];
    
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
    let asm_path = &args[2];

    let asm = match read_as_bytes(asm_path) {
      Err(e) => panic!("{}", e),
      Ok(b) => b
    };

    /* Begin parsing for real */

    // Store Option<Opcode> in an array, because the max. number is small
    // This probably only wastes about 1.5KB
    let codes = opcodes::get_opcodes();

    // The first HEADER_BYTES bytes should be a header string
    if asm.len() < HEADER_BYTES {
      println_err!("{} missing NWScript header bytes", args[2]);
      return;
    }
    println!(";;{}", std::str::from_utf8(&asm[..HEADER_BYTES]).unwrap());

    // TODO implement try!() or something for all these returns
    
    // The next 5 bytes are the T opcode - TODO get operand size from T
    let start_idx = HEADER_BYTES + 5;
    if asm.len() < start_idx {
      println_err!("{} missing NWScript size bytes", args[2]);
      return;
    }

    let longest_code = codes.iter()
      .filter_map(|c| match *c { Some(ref c) => Some(c.fmt.len()), None => None })
      .max().unwrap();
    let pad_str = String::from_utf8(repeat(0x20)
                                    .take(longest_code)
                                    .collect::<Vec<u8>>()
                                    ).unwrap();

    // T's sole operand is the file size - not sure if it's really unsigned though
    match codes.get(asm[HEADER_BYTES] as usize).and_then(|c| c.as_ref()) {
      Some(op) if op.code == 0x42 => {
        let asm_size_u32 =
          Cursor::new(&asm[HEADER_BYTES+1..start_idx]).read_u32::<BigEndian>().unwrap();
      
        if (asm_size_u32 as usize) != asm.len() {
          println_err!("T {} does not match file size ({} bytes)", asm_size_u32, asm.len());
          return;
        } else {
          // really need to set up an output stream or something
          // TODO get T.fmt etc from opcodes
          println!("T{}{:#010X}", &pad_str[0..longest_code - 1], asm_size_u32);
        }
      },
      Some(op) => {
        println_err!("Unexpected opcode {:#04X} at byte {}, expected T (0x42)",
                 op.code, HEADER_BYTES);
        return;
      },
      None => {
        println_err!("Unknown opcode {:#04X} at byte {}, expected T (0x42)",
                 asm[HEADER_BYTES], HEADER_BYTES);
        return;
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
      idx = step_or_fail!(idx, 1, asm_len); // .0 => .1, .1 => .1 + step

      // Get a command byte and interpret it
      let op = match codes.get(asm[idx.0] as usize).and_then(|c| c.as_ref()) {
        Some(op) => op,
        None => {
          println_err!("Unknown opcode {:#04X} at byte {}", asm[idx.0], idx.0);
          return;
        }
      };

      // Get the type byte - type of bytes that may be popped off the stack
      // determines legal args, but isn't necessarily the type of them
      idx = step_or_fail!(idx, 1, asm_len);
      // TODO make type an Option? To handle T etc
      let stack_type = if op.types.contains(&asm[idx.0]) {
        asm[idx.0]
      } else {
        println_err!("Type {:#04X} not in list of legal types for opcode {}",
                 asm[idx.0], op.fmt);
        return;
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
            idx = step_or_fail!(idx, size, asm_len);
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
            idx = step_or_fail!(idx, size, asm_len);
            let num = bytes_to_int(&asm[idx.0..idx.1]);

            match *arg { // wish we had fallthrough because nesting this sucks :S
              Operand::Offset(..) => print!("\t@{}", num),
              _ => print!("\t{}", num)
            };
          },
          Operand::Float(size) => {
            idx = step_or_fail!(idx, size, asm_len);
            let num = bytes_to_float(&asm[idx.0..idx.1]);
            print!("\t{}", num);
          },
          Operand::String => {
            let str_len = match prev_val {
              Some(n) => n as usize,
              None => {
                println_err!("\nString argument without preceding length argument!");
                return;
              }
            };

            idx = step_or_fail!(idx, str_len, asm_len);
            let s = std::str::from_utf8(&asm[idx.0..idx.1]).unwrap();
            print!("\t\"{}\"", s);
          
          }
          // TODO errors/streams etc
        }
      }
      println!(""); 
    }
  } else {
    println!("Usage: {} script.ldf file.ncs", args[0]);
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
