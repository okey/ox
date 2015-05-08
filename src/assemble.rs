use std;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
//use std::io::prelude::*;
use std::io::{BufRead, Result, Write, Error, ErrorKind, Seek, SeekFrom};
//use std::io::error::Error;
//use std::string::String;

use super::Routine;
use opcodes::{Opcode, NWType, get_nwtypes};

type OpcodeMap<'a> = HashMap<&'a str, &'a Opcode>;
type VariantMap<'a> = HashMap<String, (&'a Opcode, Option<&'a NWType>)>;

fn assemble_line<T: Write>(line: &String,
                           output: &mut T,
                           opcodes: &OpcodeMap,
                           variants: &VariantMap,
                           routines: Option<&HashMap<u16, Routine>>) -> Result<()> {

  let trimmed = line.trim();
  if line.starts_with(";;") || trimmed.is_empty() {
    return Ok(())
  }

  let parts:Vec<&str> = trimmed.splitn(2, ' ').map(|w| w.trim()).collect();
  let name = parts[0];

  //let vo = ;
  let (op, t_byte) = match variants.get(parts[0]) {
    Some(vo) => {
      let &(op, maybe_t) = vo;
      match maybe_t {
        Some(t) => { // regular variant, type known from variant name
          //let abbr = if t.abbr.is_some() { t.abbr.unwrap() } else { "" };
          //try!(output.write(format!("VMATCH {} {} {:#X}\n", op.fmt, abbr, op.types[0]).as_bytes()));
          (op, Some(t.code))
        },
        None => { // irregular variant, type requires additional specifier
          //try!(output.write(format!("SMATCH {}\n", op.fmt).as_bytes()));
          (op, Some(0xff)) // TODO grab next token instead
        }
      }
    },
    None => {
  //} else {
      match opcodes.get(parts[0]) {
        Some(op) => { // standard opcode, type known from name and or code
          match op.types {
            Some(ref types) => {
              match types.len() {
              // TODO change types to Option<> and skip to args if None
                //0 => { try!(output.write(format!("RMATCH {}\n", op.fmt).as_bytes())); },
                //1 => { try!(output.write(format!("RMATCH {} {}\n", op.fmt, op.types[0]).as_bytes())); },
                0 => (*op, None),
                1 => (*op, Some(types[0])),
                _ => {
                  let e_str = format!("Opcode definition error: {} has more than one type", op.fmt);
                  return Err(Error::new(ErrorKind::InvalidInput, e_str))
                }
              }
            },
            None => (*op, None)
          }
        },
        None => {
        let e_str = format!("Expected valid opcode string, got \"{}\"", parts[0]);
          return Err(Error::new(ErrorKind::InvalidInput, e_str))
        }
      }
    }
  };
  println!("{:#04X} {:?} ({})", op.code, t_byte, op.fmt);

  // technically this would allow an opcode with no type and no args, if one was defined
  // which is illegal...
  let args = match op.args {
    Some(ref args) => match t_byte {
      Some(t_byte) => match args.get(&t_byte) {
        Some(args) => args,
        None => {
          // Not sure if this is actually an error... but there are no opcodes that trigger this
          let e_str = format!("Opcode {} has no arguments for type {:#04X}", op.fmt, t_byte);
          return Err(Error::new(ErrorKind::InvalidInput, e_str))
        }
      },
      None => return Ok(())
    },
    None => return Ok(())
  };

  //let args = if t_byte.is_some() { op.args.get(t_byte.unwrap()).unwrap() } else { vec!() };

  for arg in args {
    print!("{:?}, ", arg);
    // see if there's a token, bail if not
    // then match the type of the arg and parse it
    // TODO string parsing
  }
  println!("");

  Ok(())
}

// bufread because we want lines
//#[allow(unused_variables)]
pub fn assemble<T: BufRead>(input: T,
                            opcodes: &[Option<Opcode>],
                            routines: Option<&HashMap<u16, Routine>>,
                            output_name: Option<&String>) -> Result<()> {

  let mut wtr = BufWriter::new(match output_name {
    Some(path) => box try!(File::create(path)) as Box<Write>,
    None => box std::io::stdout() as Box<Write>
  });

  // TODO don't pass output_name as an Option? Generate from input name in main if not given
  //let mut wtr = BufWriter::new(try!(File::create(path)));

  let nwtypes = get_nwtypes();
  let mut reverse_opcodes: OpcodeMap = HashMap::new();
  let mut variant_opcodes: VariantMap = HashMap::new();

  for op in opcodes.iter() {
    match op {
      &Some(ref o) => {
        reverse_opcodes.insert(&o.fmt, &o);

        match o.types {
          Some(ref types) if types.len() > 1 => {
            for t in types.iter() {
              let variant_type = match nwtypes.get(*t as usize).and_then(|c| c.as_ref()) {
                Some(t) => t,
                None => {
                  let e_str = format!("Variant type {} not found for opcode {}", t, o.fmt);
                  return Err(Error::new(ErrorKind::NotFound, e_str))
                }
              };

              // TODO what is up with EQUAL/NEQUAL and 0x24/TT?
              // TT supports all the engine types but they're allowed individually too? ????
              match variant_type.abbr {
                Some(a) => {
                  let mut variant_name = String::from_str(&o.fmt);
                  variant_name.push_str(a);

                  variant_opcodes.insert(variant_name, (&o, Some(variant_type)));
                },
                None => { variant_opcodes.insert(o.fmt.to_string(), (&o, None)); }
              };
            }
          },
          _ => ()
        }
      },
      &None => ()
    }
  }

  // Don't forget that you need to emit the header bytes, and T SIZE
  // T SIZE will needing counting bytes + a rewind.

  for line in input.lines() {
    match line {
      Ok(s) => try!(assemble_line(&s, &mut wtr, &reverse_opcodes, &variant_opcodes, routines)),
      Err(reason) => return Err(reason)
    }
  }

  /*let index = try!(wtr.seek(SeekFrom::Start(0)));
  if index != 0 {
    println!("wtf");
  }
  try!(wtr.write(b"overwritten!"));*/

  return Ok(())
}
