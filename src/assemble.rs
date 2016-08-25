use std;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufWriter;
//use std::io::prelude::*;
use std::io;
use std::num;
use std::io::{BufRead, Write, ErrorKind, Seek, SeekFrom};
use byteorder;
use byteorder::{BigEndian, WriteBytesExt};
//use std::io::error::Error;
//use std::string::String;

use std::str::FromStr;
use std::error::Error;

use super::Routine;
use opcodes::{Opcode, NWType, get_nwtypes, Operand, OpcodeE, NWTypeE};

#[derive(Debug)]
pub enum AssemblyError {
  ParseError(String), // TODO line count? Also probably could be str instead of String?
  IOError(io::Error)
}

impl From<io::Error> for AssemblyError {
  fn from(e: io::Error) -> Self {
    AssemblyError::IOError(e)
  }
}

impl From<num::ParseIntError> for AssemblyError {
  fn from(e: num::ParseIntError) -> Self {
    AssemblyError::ParseError(e.description().to_string())
  }
}

impl From<num::ParseFloatError> for AssemblyError {
  fn from(e: num::ParseFloatError) -> Self {
    AssemblyError::ParseError(e.description().to_string())
  }
}

pub type AssemblyResult = Result<(), AssemblyError>;

type OpcodeMap<'a> = HashMap<String, &'a Opcode>;
type VariantMap<'a> = HashMap<String, (&'a Opcode, Option<&'a NWType>)>;
type RoutineMap<'a> = HashMap<&'a String, &'a Routine>;

fn float_str_to_bytes(size: usize, s: &str) -> Result<Vec<u8>, AssemblyError> {
  let mut buf = vec!();
  match size {
    4 => {
      let num = try!(f32::from_str(s));
      try!(buf.write_f32::<BigEndian>(num));
    },
    _ => {
      let msg = format!("Unsupported float size {} bytes", size);
      return Err(AssemblyError::ParseError(msg.to_string()));
    }
  }
  Ok(buf)
}

fn int_str_to_bytes(size: usize, pfx: Option<&str>,
                     base: u32, s: &str) -> Result<Vec<u8>, AssemblyError> {
  let offset = match pfx {
    Some(p) => if s.starts_with(p) { p.len() } else { 0 },
    None => 0
  };
  let mut buf = vec!();
  match size {
    2 => {
      let num = try!(i16::from_str_radix(&s[offset..], base));
      try!(buf.write_i16::<BigEndian>(num));
    },
    4 => {
      let num = try!(i32::from_str_radix(&s[offset..], base));
      try!(buf.write_i32::<BigEndian>(num));
    },
    _ => {
      let msg = format!("Unsupported int size {} bytes", size);
      return Err(AssemblyError::ParseError(msg.to_string()));
    }
  }
  Ok(buf)
}

fn uint_str_to_bytes(size: usize, pfx: Option<&str>,
                     base: u32, s: &str) -> Result<Vec<u8>, AssemblyError> {
  // what happens if pfx is ""? always fail??? or awlays match???
  let offset = match pfx {
    Some(p) => if s.starts_with(p) { p.len() } else { 0 },
    None => 0
  };
  let mut buf = vec!();
  // seems like this whole thing could be a macro? maybe? some of it, at least
  match size {
    1 => {
      let num = try!(u8::from_str_radix(&s[offset..], base));
      try!(buf.write_u8(num));
    },
    2 => {
      let num = try!(u16::from_str_radix(&s[offset..], base));
      try!(buf.write_u16::<BigEndian>(num));
    },
    4 => {
      let num = try!(u32::from_str_radix(&s[offset..], base));
      try!(buf.write_u32::<BigEndian>(num));
    },
    _ => {
      let msg = format!("Unsupported uint size {} bytes", size);
      return Err(AssemblyError::ParseError(msg.to_string()));
    }
  }
  Ok(buf)
}

// TODO return Vec<u8> instead
// TODO correctly take size into account and return only that many bytes
fn parse_arg(o: &Operand, s: &str, routines: &RoutineMap) -> Result<Vec<u8>, AssemblyError> {
  let mut buf = vec!();
  match *o {
    Operand::Object(sz) | Operand::Size(sz) => {
      buf.extend(try!(uint_str_to_bytes(sz, Some("0x"), 16, s)));
      /*
      let offset = if s.starts_with("0x") { 2 } else { 0 }; // TODO handle prefixes properly
      let num = try!(u32::from_str_radix(&s[offset..], 16)); // TODO variable size
      print!(" {:#010X}", num);
      try!(buf.write_u32::<BigEndian>(num)); //will this swap the bytes twice on LE?
      */
    },
    Operand::Routine(sz) => { // TODO handle #mode
      let parts: Vec<&str> = s.split('#').collect();
      let name = parts[0];

      print!(" ~{}~", name);
      match routines.get(&name.to_string()) {
        Some(rtn) => {
          print!(" {:#X}", rtn.code);
        },
        None => () // TODO throw error if not in #mode
      }

      if parts.len() == 2 {
        let explicit = parts[1];
        /*let offset = if explicit.starts_with("0x") { 2 } else { 0 };
        let num = try!(u32::from_str_radix(&explicit[offset..], 16));
        print!(" # {:#X}", num);
        try!(buf.write_u32::<BigEndian>(num));*/
        buf.extend(try!(uint_str_to_bytes(sz, Some("0x"), 16, explicit)));
      } // TODO fail if len > 2
    },
    Operand::ArgCount(sz) => {
      buf.extend(try!(uint_str_to_bytes(sz, None, 10, s)));
    },
    Operand::Offset(sz) => {
      /*let offset = if s.starts_with("@") { 1 } else { 0 };
      let num = try!(i32::from_str(&s[offset..]));
      print!(" @{}", num);
      try!(buf.write_i32::<BigEndian>(num));*/
      buf.extend(try!(int_str_to_bytes(sz, Some("@"), 10, s)));
    },
    Operand::Integer(sz)  => {
      /*let num = try!(i32::from_str(s));
      print!(" {}", num);
      try!(buf.write_i32::<BigEndian>(num));*/
      buf.extend(try!(int_str_to_bytes(sz, None, 10, s)));
    },
    Operand::Float(sz) => {
      /*let num = try!(f32::from_str(s));
      print!(" {}", num);
      try!(buf.write_f32::<BigEndian>(num));*/
      buf.extend(try!(float_str_to_bytes(sz, s)));
    },
    Operand::String => {
      let len = s.len() - 2; // TODO clean up this
      print!(" {:#04X} {}", len, &s[1..s.len()-1]); // TODO process escape characters
      try!(buf.write_u16::<BigEndian>(len as u16));
      buf.extend(s[1..s.len()-1].as_bytes())
    },
  }
  Ok(buf)
}

fn split_line<'a>(line: &'a String) -> Result<Vec<&'a str>, AssemblyError> {
  let mut result = vec!();
  let mut stack: Vec<char> = vec!();
  let mut escape = false;
  let mut start = 0;

  if line.starts_with(";;") {
    return Ok(result);
  }

  for (n, c) in line.char_indices() { // chars and byte indices - not sure if codepoints though
    if escape {
      escape = false;
      continue;
    }
    match c {
      // will need an output pass to substitute escape characters and calculate right length
      '\\' => { escape = true; },
      '\"' => { // using ends_with means putting c in a temp vector :C
        if stack.len() > 0 && stack[stack.len()-1] == c {
          stack.pop();
        } else {
          stack.push(c);
          // TODO strip ""
          // TODO make sure a string is a token?
          // TODO deal with r""?
          continue;
        }
      },
      _ if stack.len() > 0 => (),
      _ if c.is_whitespace() && n > start => {
        result.push(&line[start..n]); // I hope this slices bytes..
        start = n + 1;
      },
      _ if c.is_whitespace() => { start = n + 1; },
      _ => ()
    }
  }
  if start < line.len() {
    result.push(&line[start..]);
  }
  if stack.len() > 0 {
    Err(AssemblyError::ParseError(format!("Line with unclosed delimiter: {}", line).to_string()))
  } else {
    Ok(result)
  }
}

// TODO return a Vec<u8> or something
// TODO assert first opcode is T
fn assemble_line<T: Write>(line: &String,
                           output: &mut T,
                           opcodes: &OpcodeMap,
                           variants: &VariantMap,
                           routines: &RoutineMap) -> AssemblyResult {

  let parts = try!(split_line(line));
  if parts.len() == 0 {
    return Ok(()); // skip
  }
  /*if parts.len() == 1 {
    return Err(AssemblyError::ParseError(format!("Illegal line: {}", line).to_string()));
  }*/
  let name = &String::from(parts[0]); // TODO replace with .into_string()
  let mut tokens = 1;
  //println!("{:?}", parts);

  let (op, t_byte) = match variants.get(name) {
    Some(vo) => {
      let &(op, maybe_t) = vo;
      match maybe_t {
        // regular variant, type known from variant name
        Some(t) => { (op, Some(t.code as u8)) },
        // irregular variant, type requires additional specifier
        None if parts.len() > 1 => match op.types {
          Some(ref optypes) => match optypes.get(try!(u8::from_str(parts[1])) as usize) {
            Some(tt) => {
              tokens += 1;
              (op, Some(*tt))
            },
            None => {
              let e = format!("Opcode {} with illegal or unknown type specifier", op.code);
              return Err(AssemblyError::ParseError(e.to_string()));
            }
          },
          // No types expected for variant - can this even happen? indicative of bad data design?
          None => (op, None)
        },
        // No name variant and no explicit type, so fail
        None => {
          let e = format!("Opcode with no type specifier: {}", op.code);
          return Err(AssemblyError::ParseError(e.to_string()));
        }
      }
    },
    //1 => { try!(output.write(format!("RMATCH {} {}\n", op.code, op.types[0]).as_bytes())); },
    None => {
      match opcodes.get(name) {
        // standard opcode, type known from name and or code
        Some(op) => {
          match op.types {
            // Normal type
            Some(ref types) => {
              match types.len() {
                0 => (*op, None),
                1 => (*op, Some(types[0])),
                _ => { // ambiguous
                  let msg = format!("Opcode definition error: {} has more than one type", op.code);
                  return Err(AssemblyError::IOError(io::Error::new(ErrorKind::InvalidInput, msg)))
                }
              }
            },
            // T
            None => (*op, None)
          }
        },
        None => {
        let e_str = format!("Expected valid opcode string, got \"{}\"", name);
          return Err(AssemblyError::IOError(io::Error::new(ErrorKind::InvalidInput, e_str)))
        }
      }
    }
  };
  let real_t = match t_byte {
    None => "     ".to_string(),
    Some(b) => format!("{:#04X} ", b)
  };
  print!("{:#04X} ({})\t{}({:?})", op.code, op.code, real_t, t_byte);

  let mut buf = vec!();
  try!(buf.write_u8(op.code as u8));
  match t_byte {
    None => (),
    //None => try!(output.write_u32::<BigEndian>(num));))
    Some(b) => { try!(buf.write_u8(b)); }
  }
  try!(output.write(buf.as_slice()));

  // technically this would allow an opcode with no type and no args, if one was defined..
  // no such code exists, however, and it wouldn't really make sense...
  let args = match op.args {
    Some(ref args) => match t_byte {
      Some(t_byte) => match args.get(&t_byte) {
        Some(args) => {
          let mut new_args = vec!();
          for (n, a) in args.iter().enumerate() {
            match a {
              &Operand::Size(..) => match args.get(n+1) {
                Some(p) => match p {
                  &Operand::String => { continue; }, // don't expect string length to be specified
                  _ => { new_args.push(a) }
                },
                None => { new_args.push(a); }
              },
              _ => { new_args.push(a); }
            }
          }
          new_args
        },
        None => {
          // Not sure if this is actually an error... but there are no opcodes that trigger this
          let e_str = format!("Opcode {} has no arguments for type {:#04X}", op.code, t_byte);
          return Err(AssemblyError::IOError(io::Error::new(ErrorKind::InvalidInput, e_str)))
        }
      },
      None => if op.code == OpcodeE::T { // hack for T
        args.get(&(0x00 as u8)).unwrap().iter().map(|c| c).collect()
      } else {
        println!("");
        return Ok(())
      }
    },
    None => {
      println!("");
      return Ok(())
    }
  };

  if args.len() + tokens != parts.len() {
    let e_str = format!("Opcode {} expects {} args, got {}", op.code, args.len(),
                        parts.len() - tokens);
    return Err(AssemblyError::ParseError(e_str))
  }

  // int types have to_be() for big endian conversion!

  // TODO function?
  for (n, arg) in args.iter().enumerate() {
    let idx = tokens + n;
    let bytes = try!(parse_arg(*arg, parts[idx], routines));
    try!(output.write(bytes.as_slice()));
    /*match **arg {
      Operand::Size(sz) if sz == 4 => {
        // TODO reliably skip hex
        let num = try!(usize::from_str_radix(&parts[idx][2..], 16));
        print!(" {:#010X}", num);
        ()
      }
      _ => ()
    }*/
    print!(" {:?},", arg);
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
                            output_name: Option<&String>) -> AssemblyResult {

  let mut wtr = BufWriter::new(match output_name {
    Some(path) => box try!(File::create(path)) as Box<Write>,
    None => box std::io::stdout() as Box<Write>
  });

  // TODO don't pass output_name as an Option? Generate from input name in main if not given
  //let mut wtr = BufWriter::new(try!(File::create(path)));

  let nwtypes = get_nwtypes();
  let mut reverse_opcodes: OpcodeMap = HashMap::new();
  let mut variant_opcodes: VariantMap = HashMap::new();

  for op in opcodes {
    match op {
      &Some(ref o) => {
        reverse_opcodes.insert(o.code.to_string(), &o);

        match o.types {
          Some(ref types) if types.len() > 1 => {
            for t in types.iter() {
              let variant_type = match nwtypes.get(*t as usize).and_then(|c| c.as_ref()) {
                Some(t) => t,
                None => {
                  let e_str = format!("Variant type {} not found for opcode {}", t, o.code);
                  return Err(AssemblyError::IOError(io::Error::new(ErrorKind::NotFound, e_str)))
                }
              };

              // TODO what is up with EQUAL/NEQUAL and 0x24/TT?
              // TT supports all the engine types but they're allowed individually too? ????
              match variant_type.abbr {
                Some(a) => {
                  // TODO FIXME use try! instead???
                  let mut variant_name = o.code.to_string();
                  variant_name.push_str(a);

                  variant_opcodes.insert(variant_name, (&o, Some(variant_type)));
                },
                None => { variant_opcodes.insert(o.code.to_string(), (&o, None)); }
              };
            }
          },
          _ => ()
        }
      },
      &None => ()
    }
  }

  let mut reverse_routines: RoutineMap = HashMap::new();
  match routines {
    Some(routines) => for rtn in routines.values() {
      reverse_routines.insert(&rtn.name, rtn);
    },
    None => ()
  }


  // Don't forget that you need to emit the header bytes, and T SIZE
  // T SIZE will needing counting bytes + a rewind.
  try!(wtr.write(b"NCS V1.0")); // fake header

  for line in input.lines() {
    match line {
      Ok(s) => try!(assemble_line(&s, &mut wtr,
                                  &reverse_opcodes, &variant_opcodes, &reverse_routines)),
      Err(reason) => return Err(AssemblyError::IOError(reason))
    }
  }

  /*let index = try!(wtr.seek(SeekFrom::Start(0)));
  if index != 0 {
    println!("wtf");
  }
  try!(wtr.write(b"overwritten!"));*/

  return Ok(())
}
