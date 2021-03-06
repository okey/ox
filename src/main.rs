extern crate docopt;
extern crate byteorder;
#[macro_use]
extern crate serde_derive;


use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::prelude::*;
use std::string::String;

mod macros;
mod opcodes;
mod io_utils;
mod disassemble;
mod assemble;
mod nwscript {
    include!(concat!(env!("OUT_DIR"), "/nwscript.rs"));
}

use docopt::Docopt;
use io_utils::read_as_string;
use disassemble::{disassemble, DisassemblyError};
use assemble::AssemblyError;
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

const USAGE: &'static str = "
Usage: ox d <input> -c <def.ldf> [--nwn] [-o <output.ox>]
       ox a <input> [-c <def.ldf> [--nwn]] [-o <output.ncs>]
       ox --help

Options:
  d <input.ox>            Disassemble input.ncs file.
  a <input.ncs>           Assemble input.ox file.

  -c, --define DFILE      Engine routine definition file.
  --nwn                   Expect NWN-style routine definitions.
  -o, --output OUTPUT     The file to write output to.
  -h, --help              Show this message.
";

#[derive(Debug, Deserialize)]
struct Args {
  cmd_d: bool,
  cmd_a: bool,
  arg_input: String,
  flag_define: String,
  flag_output: String,
  flag_nwn: bool,
}

// gold-plating: tabs/spaces, hex options, cyclic (-r?) option that is -d then -a or vice versa

// -c is poorly named, and -d and --define are easily confused. TODO fix this.

// TODO config via config struct???

fn main() {
  let args: Args = Docopt::new(USAGE)
    .and_then(|d| d.deserialize())
    .unwrap_or_else(|e| e.exit());

  let opcodes = opcodes::get_opcodes();

  let doc = if args.flag_define.len() > 0 {
    let def_path = &args.flag_define;

    // Read the definitions file
    let res = match read_as_string(def_path) {
      Err(e) => panic!("{}", e),
      Ok(s) => s
    };

    // Parse definitions with peg
    match document(res.as_ref()) { // TODO nwn mode
      Err(e) => panic!("{}", e),
      Ok(d) => Some(d)
    }
  } else {
    None
  };

  // Assemble
  if args.cmd_a {
    let asm_path = &args.arg_input;
    let output_path = if "" == args.flag_output { None } else { Some(&args.flag_output) };

    let rdr = std::io::BufReader::new(match File::open(asm_path){
      Ok(f) => f,
      Err(reason) => panic!("Opening {} failed: {}", asm_path, Error::description(&reason))
    });

    if doc.is_some() {
      let (_, routines) = build_tables(doc.unwrap());

      match assemble::assemble(rdr, &opcodes, Some(&routines), output_path) {
        Ok(_) => println!("Assembly complete, no defs"),
        Err(e) => match e {
          AssemblyError::ParseError(m) => panic!("Assembly failed: {}", m),
          AssemblyError::IOError(e) => panic!("Assembly failed: {}", e),
          // TODO fix I/O error handling
        }
      }
    } else {
      match assemble::assemble(rdr, &opcodes, None, output_path) {
        Ok(_) => println!("Assembly complete, no defs"),
        Err(e) => match e {
          AssemblyError::ParseError(m) => panic!("Assembly failed: {}", m),
          AssemblyError::IOError(e) => panic!("Assembly failed: {}", e),
          // TODO fix I/O error handling
        }
      }
    }

    return
  }

  // Disassemble
  if args.cmd_d {
    let output_path = if "" == args.flag_output { None } else { Some(&args.flag_output) };

    // Build tables
    let (constants, routines) = build_tables(doc.unwrap());
    // TODO stick this at the front of the writer? pass the writer in to fn instead?
    println!(";;Read {} constants and {} routines", constants.len(), routines.len());

    // Read the compiled file
    let asm_path = &args.arg_input; // TODO stream this instead
    let mut rdr = std::io::BufReader::new(match File::open(&asm_path){
      Ok(f) => f,
      Err(reason) => panic!("Opening {} failed: {}", &asm_path, Error::description(&reason))
    });

    match disassemble(&mut rdr, &opcodes, &routines, output_path) {
      Ok(_) => (),
      Err(e) => match e {
        DisassemblyError::OpStreamError(m, b) => panic!("Disassembly failed: {} (byte {})", m, b),
        _ => panic!("Disassembly failed ???") // TODO fix error handling
      }
    }

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
