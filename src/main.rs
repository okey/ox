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
use std::io::prelude::*;
use std::string::String;

mod macros;
mod opcodes;
mod io_utils;
mod disassemble;

use io_utils::{read_as_bytes, read_as_string};
use disassemble::disassemble;

peg_file! nwscript("nwscript.rustpeg");
use nwscript::document;


#[derive(Debug)]
pub struct Constant {
  type_name: String,
  name: String,
  value: String
}

#[derive(Debug)]
pub struct Routine {
  return_type: String,
  name: String,
  code: u16,
  args: Vec<RoutineArg>
}

#[derive(Debug)]
pub struct RoutineArg {
  type_name: String,
  name: String,
  default_value: Option<String>
}
#[derive(Debug)]
pub enum Statement {
  Routine(Routine),
  Constant(Constant)
}

fn build_tables(list: Vec<Statement>) -> (HashMap<String, Constant>, HashMap<u16, Routine>) {
  let mut constants = HashMap::new();
  let mut commands = HashMap::new(); // 16-bit, not sure if int or uint

  for st in list {
    match st {
      Statement::Constant(c) => {
        match constants.insert(c.name.clone(), c) {
          Some(c) => {
            let d = constants.get(&c.name).unwrap();
            println_err!("Error: Multiple declarations of variable {}", d.name);
            println_err!("     > {} {} = {};", d.type_name.trim(), d.name, d.value);
            panic!("duplicate variable");
          },
          None => ()
        }
      },
      Statement::Routine(c) => {
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
