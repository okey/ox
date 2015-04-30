use std::iter::repeat;
use std::collections::HashMap;

use self::Operand::*;


#[derive(Debug)]
pub struct Opcode {
  pub code: u8,
  pub fmt: &'static str,
  pub types: Vec<u8>,
  pub args: Option<HashMap<u8, Vec<Operand>>>
}

// These all have (potentially) different formatting requirements
#[derive(Debug)]
pub enum Operand {
  Routine(usize),
  Object(usize),
  Size(usize),
  Offset(usize),
  Integer(usize),
  Float(usize),
  String,
  ArgCount(usize)
}

/*
// wrapping Operand like this might make the matches cleaner?
pub enum OperandC {
  Signed(Operand, usize),
  Unsigned(Operand, usize),
  Float(usize),
  String
}*/


macro_rules! opcode {
  ($c:expr, $f:expr, $t:expr, $a:expr) => {Opcode { code: $c, fmt: $f, types: $t, args: $a}}
}

macro_rules! hashmap {
    ($( $key: expr => $val: expr ),*) => {{
         let mut map = HashMap::new();
         $( map.insert($key, $val); )*
         map
    }}
}


#[allow(dead_code)] // bug in Rust with constants that are not used in impls
const MAX_OPCODES: usize = 256;


pub fn get_opcodes() -> Box<[Option<Opcode>]> {
  // Take N * None without Clone or Copy
  let mut x:Vec<Option<Opcode>> = repeat(true).take(MAX_OPCODES).map(|_| None).collect();
  
  // TODO fix python generation of codes for new format
  // TODO auto insertion of generated values?
  // TODO automatically check for duplicate keys

  x[0x01] = Some(Opcode{ code: 0x01, fmt: "CPDOWNSP",
                         types: vec!(0x01),
                         args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[0x02] = Some(Opcode{ code: 0x02, fmt: "RSADD*",
                         types: vec!(0x03, 0x04, 0x05, 0x06), args: None });
  x[0x03] = Some(Opcode{ code: 0x03, fmt: "CPTOPSP",
                         types: vec!(0x01),
                         args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[0x04] = Some(Opcode{ code: 0x04, fmt: "CONST*",
                         types: vec!(0x03, 0x04, 0x05, 0x06),
                         args: Some(hashmap!(0x03 => vec!(Integer(4)),
                                             0x04 => vec!(Float(4)),
                                             0x05 => vec!(Size(2), String),
                                             0x06 => vec!(Object(4)))) });
  x[0x05] = Some(Opcode{ code: 0x05, fmt: "ACTION",
                         types: vec!(0x00),
                         args: Some(hashmap!(0x00 => vec!(Routine(2), ArgCount(1)))) });
  x[0x06] = Some(Opcode{ code: 0x06, fmt: "LOGANDII",
                         types: vec!(0x20), args: None });
  x[0x07] = Some(Opcode{ code: 0x07, fmt: "LOGORII",
                         types: vec!(0x20), args: None });
  x[0x08] = Some(Opcode{ code: 0x08, fmt: "INCORII",
                         types: vec!(0x20), args: None });
  x[0x09] = Some(Opcode{ code: 0x09, fmt: "EXCORII",
                         types: vec!(0x20), args: None });
  x[0x0A] = Some(Opcode{ code: 0x0A, fmt: "BOOLANDII",
                         types: vec!(0x20), args: None });
  x[0x0B] = Some(Opcode{ code: 0x0B, fmt: "EQUAL**",
                         types: vec!(0x20, 0x21, 0x22, 0x23, 0x30, 0x31, 0x32,
                                     0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39),
                         args: Some(hashmap!(0x20 => vec!(), 0x21 => vec!(), 0x22 => vec!(),
                                             0x23 => vec!(), 0x30 => vec!(), 0x31 => vec!(),
                                             0x32 => vec!(), 0x33 => vec!(), 0x34 => vec!(),
                                             0x35 => vec!(), 0x36 => vec!(), 0x37 => vec!(),
                                             0x38 => vec!(), 0x39 => vec!(),
                                             0x24 => vec!(Size(2)))) });
  x[0x0C] = Some(Opcode{ code: 0x0C, fmt: "NEQUAL**",
                         types: vec!(0x20, 0x21, 0x22, 0x23, 0x30, 0x31, 0x32,
                                     0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39),
                         args: Some(hashmap!(0x20 => vec!(), 0x21 => vec!(), 0x22 => vec!(),
                                             0x23 => vec!(), 0x30 => vec!(), 0x31 => vec!(),
                                             0x32 => vec!(), 0x33 => vec!(), 0x34 => vec!(),
                                             0x35 => vec!(), 0x36 => vec!(), 0x37 => vec!(),
                                             0x38 => vec!(), 0x39 => vec!(),
                                             0x24 => vec!(Size(2)))) });
  x[0x0D] = Some(Opcode{ code: 0x0D, fmt: "GEQ**",
                         types: vec!(0x20, 0x21), args: None });
  x[0x0E] = Some(Opcode{ code: 0x0E, fmt: "GT**",
                         types: vec!(0x20, 0x21), args: None });
  x[0x0F] = Some(Opcode{ code: 0x0F, fmt: "LT**",
                         types: vec!(0x20, 0x21), args: None });
  x[0x10] = Some(Opcode{ code: 0x10, fmt: "LEQ**",
                         types: vec!(0x20, 0x21), args: None });
  x[0x11] = Some(Opcode{ code: 0x11, fmt: "SHLEFTII",
                         types: vec!(0x20), args: None });
  x[0x12] = Some(Opcode{ code: 0x12, fmt: "SHRIGHTII",
                         types: vec!(0x20), args: None });
  x[0x13] = Some(Opcode{ code: 0x13, fmt: "USHRIGHTII",
                         types: vec!(0x20), args: None });
  x[0x14] = Some(Opcode{ code: 0x14, fmt: "ADD**",
                         types: vec!(0x20, 0x25, 0x26, 0x21, 0x23, 0x3a), args: None });
  x[0x15] = Some(Opcode{ code: 0x15, fmt: "SUB**",
                         types: vec!(0x20, 0x25, 0x26, 0x21, 0x3a), args: None });
  x[0x16] = Some(Opcode{ code: 0x16, fmt: "MUL**",
                         types: vec!(0x20, 0x25, 0x26, 0x21, 0x3b, 0x3c), args: None });
  x[0x17] = Some(Opcode{ code: 0x17, fmt: "DIV**",
                         types: vec!(0x20, 0x25, 0x26, 0x21, 0x3b), args: None });
  x[0x18] = Some(Opcode{ code: 0x18, fmt: "MODII",
                         types: vec!(0x20), args: None });
  x[0x19] = Some(Opcode{ code: 0x19, fmt: "NEG*",
                         types: vec!(0x03, 0x04), args: None });
  x[0x1A] = Some(Opcode{ code: 0x1A, fmt: "COMPI",
                         types: vec!(0x03), args: None });
  x[0x1B] = Some(Opcode{ code: 0x1B, fmt: "MOVSP",
                         types: vec!(0x00), args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[0x1C] = Some(Opcode{ code: 0x1C, fmt: "STORE_STATEALL",
                         types: vec!(0x08), args: None });
  x[0x1D] = Some(Opcode{ code: 0x1D, fmt: "JMP",
                         types: vec!(0x00), args: Some(hashmap!(0x00 => vec!(Offset(4)))) });;
  x[0x1E] = Some(Opcode{ code: 0x1E, fmt: "JSR",
                         types: vec!(0x00), args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[0x1F] = Some(Opcode{ code: 0x1F, fmt: "JZ",
                         types: vec!(0x00), args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[0x20] = Some(Opcode{ code: 0x20, fmt: "RETN",
                         types: vec!(0x00), args: None });
  x[0x21] = Some(Opcode{ code: 0x21, fmt: "DESTRUCT",
                         types: vec!(0x01),
                         args: Some(hashmap!(0x01 => vec!(Size(2), Offset(2), Size(2)))) });
  x[0x22] = Some(Opcode{ code: 0x22, fmt: "NOTI",
                         types: vec!(0x03), args: None });
  x[0x23] = Some(Opcode{ code: 0x23, fmt: "DECISP",
                         types: vec!(0x03), args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[0x24] = Some(Opcode{ code: 0x24, fmt: "INCISP",
                         types: vec!(0x03), args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[0x25] = Some(Opcode{ code: 0x25, fmt: "JNZ",
                         types: vec!(0x00), args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[0x26] = Some(Opcode{ code: 0x26, fmt: "CPDOWNBP",
                         types: vec!(0x01),
                         args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[0x27] = Some(Opcode{ code: 0x27, fmt: "CPTOPBP",
                         types: vec!(0x01),
                         args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[0x28] = Some(Opcode{ code: 0x28, fmt: "DECIBP",
                         types: vec!(0x03), args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[0x29] = Some(Opcode{ code: 0x29, fmt: "INCIBP",
                         types: vec!(0x03), args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[0x2A] = Some(Opcode{ code: 0x2A, fmt: "SAVEBP",
                         types: vec!(0x00), args: None });
  x[0x2B] = Some(Opcode{ code: 0x2B, fmt: "RESTOREBP",
                         types: vec!(0x00), args: None });
  x[0x2C] = Some(Opcode{ code: 0x2C, fmt: "STORE_STATE",
                         types: vec!(0x10),
                         args: Some(hashmap!(0x10 => vec!(Size(4), Size(4)))) });
  x[0x2D] = Some(Opcode{ code: 0x2D, fmt: "NOP",
                         types: vec!(0x00), args: None });
  x[0x42] = Some(Opcode{ code: 0x42, fmt: "T",
                         types: vec!(), args: None });

  // Does this actually need to be boxed???
  x.into_boxed_slice()
}
