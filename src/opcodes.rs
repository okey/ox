use std::iter::repeat;
use std::collections::HashMap;

use self::Operand::*;


#[derive(Debug)]
pub struct Opcode {
  pub code: u8,
  pub fmt: &'static str,
  pub types: Option<Vec<u8>>,
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

// this would need data extraction again... maybe that's okay if we just read raw bytes?
// it's not like an op can ever be invalid if it has the correct number of byes
// it will mean matching on code and _type twice, though
// but unless I use enums for code and _type and have a 1:1 type mapping from
// (code, _type) => argtype I'll have to match on code and _type again anyway...
pub struct OpPayload<'a > {
  pub start: usize,
  pub code: &'a Opcode, //dodgy, maybe should make an enum.. or pass a reference?
  pub _type: Option<u8>, // could be an NWType?
  pub args: Vec<(&'a Operand, Vec<u8>)>// maybe everything should stay as bytes until we print?
}// Too bad we can't have a struct hack like in C?
// Maybe I should alter the


pub struct NWType {
  pub code: u8,
  pub abbr: Option<&'static str>,
  pub desc: &'static str
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
const MAX_OPCODES: usize = 256; // 0xFF
const MAX_STYPES: usize = 61; // 0x3D


pub fn get_opcodes() -> Box<[Option<Opcode>]> {
  // Take N * None without Clone or Copy
  let mut x:Vec<Option<Opcode>> = repeat(true).take(MAX_OPCODES).map(|_| None).collect();

  // TODO fix python generation of codes for new format
  // TODO auto insertion of generated values?
  // TODO automatically check for duplicate keys

  x[0x01] = Some(Opcode{ code: 0x01, fmt: "CPDOWNSP",
                         types: Some(vec!(0x01)),
                         args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[0x02] = Some(Opcode{ code: 0x02, fmt: "RSADD",
                         types: Some(vec!(0x03, 0x04, 0x05, 0x06)), args: None });
  x[0x03] = Some(Opcode{ code: 0x03, fmt: "CPTOPSP",
                         types: Some(vec!(0x01)),
                         args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[0x04] = Some(Opcode{ code: 0x04, fmt: "CONST",
                         types: Some(vec!(0x03, 0x04, 0x05, 0x06)),
                         args: Some(hashmap!(0x03 => vec!(Integer(4)),
                                             0x04 => vec!(Float(4)),
                                             0x05 => vec!(Size(2), String),
                                             0x06 => vec!(Object(4)))) });
  x[0x05] = Some(Opcode{ code: 0x05, fmt: "ACTION",
                         types: Some(vec!(0x00)),
                         args: Some(hashmap!(0x00 => vec!(Routine(2), ArgCount(1)))) });
  x[0x06] = Some(Opcode{ code: 0x06, fmt: "LOGANDII",
                         types: Some(vec!(0x20)), args: None });
  x[0x07] = Some(Opcode{ code: 0x07, fmt: "LOGORII",
                         types: Some(vec!(0x20)), args: None });
  x[0x08] = Some(Opcode{ code: 0x08, fmt: "INCORII",
                         types: Some(vec!(0x20)), args: None });
  x[0x09] = Some(Opcode{ code: 0x09, fmt: "EXCORII",
                         types: Some(vec!(0x20)), args: None });
  x[0x0A] = Some(Opcode{ code: 0x0A, fmt: "BOOLANDII",
                         types: Some(vec!(0x20)), args: None });
  x[0x0B] = Some(Opcode{ code: 0x0B, fmt: "EQUAL",
                         types: Some(vec!(0x20, 0x21, 0x22, 0x23, 0x024, 0x30, 0x31,
                                     0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39)),
                         args: Some(hashmap!(0x20 => vec!(), 0x21 => vec!(), 0x22 => vec!(),
                                             0x23 => vec!(), 0x24 => vec!(), 0x30 => vec!(),
                                             0x31 => vec!(), 0x32 => vec!(), 0x33 => vec!(),
                                             0x34 => vec!(), 0x35 => vec!(), 0x36 => vec!(),
                                             0x37 => vec!(), 0x38 => vec!(), 0x39 => vec!(),
                                             0x24 => vec!(Size(2)))) });
  x[0x0C] = Some(Opcode{ code: 0x0C, fmt: "NEQUAL",
                         types: Some(vec!(0x20, 0x21, 0x22, 0x23, 0x30, 0x31, 0x32,
                                     0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39)),
                         args: Some(hashmap!(0x20 => vec!(), 0x21 => vec!(), 0x22 => vec!(),
                                             0x23 => vec!(), 0x24 => vec!(), 0x30 => vec!(),
                                             0x31 => vec!(), 0x32 => vec!(), 0x33 => vec!(),
                                             0x34 => vec!(), 0x35 => vec!(), 0x36 => vec!(),
                                             0x37 => vec!(), 0x38 => vec!(), 0x39 => vec!(),
                                             0x24 => vec!(Size(2)))) });
  x[0x0D] = Some(Opcode{ code: 0x0D, fmt: "GEQ",
                         types: Some(vec!(0x20, 0x21)), args: None });
  x[0x0E] = Some(Opcode{ code: 0x0E, fmt: "GT",
                         types: Some(vec!(0x20, 0x21)), args: None });
  x[0x0F] = Some(Opcode{ code: 0x0F, fmt: "LT",
                         types: Some(vec!(0x20, 0x21)), args: None });
  x[0x10] = Some(Opcode{ code: 0x10, fmt: "LEQ",
                         types: Some(vec!(0x20, 0x21)), args: None });
  x[0x11] = Some(Opcode{ code: 0x11, fmt: "SHLEFTII",
                         types: Some(vec!(0x20)), args: None });
  x[0x12] = Some(Opcode{ code: 0x12, fmt: "SHRIGHTII",
                         types: Some(vec!(0x20)), args: None });
  x[0x13] = Some(Opcode{ code: 0x13, fmt: "USHRIGHTII",
                         types: Some(vec!(0x20)), args: None });
  x[0x14] = Some(Opcode{ code: 0x14, fmt: "ADD",
                         types: Some(vec!(0x20, 0x25, 0x26, 0x21, 0x23, 0x3a)), args: None });
  x[0x15] = Some(Opcode{ code: 0x15, fmt: "SUB",
                         types: Some(vec!(0x20, 0x25, 0x26, 0x21, 0x3a)), args: None });
  x[0x16] = Some(Opcode{ code: 0x16, fmt: "MUL",
                         types: Some(vec!(0x20, 0x25, 0x26, 0x21, 0x3b, 0x3c)), args: None });
  x[0x17] = Some(Opcode{ code: 0x17, fmt: "DIV",
                         types: Some(vec!(0x20, 0x25, 0x26, 0x21, 0x3b)), args: None });
  x[0x18] = Some(Opcode{ code: 0x18, fmt: "MODII",
                         types: Some(vec!(0x20)), args: None });
  x[0x19] = Some(Opcode{ code: 0x19, fmt: "NEG",
                         types: Some(vec!(0x03, 0x04)), args: None });
  x[0x1A] = Some(Opcode{ code: 0x1A, fmt: "COMPI",
                         types: Some(vec!(0x03)), args: None });
  x[0x1B] = Some(Opcode{ code: 0x1B, fmt: "MOVSP",
                         types: Some(vec!(0x00)), args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[0x1C] = Some(Opcode{ code: 0x1C, fmt: "STORE_STATEALL",
                         types: Some(vec!(0x08)), args: None });
  x[0x1D] = Some(Opcode{ code: 0x1D, fmt: "JMP",
                         types: Some(vec!(0x00)), args: Some(hashmap!(0x00 => vec!(Offset(4)))) });;
  x[0x1E] = Some(Opcode{ code: 0x1E, fmt: "JSR",
                         types: Some(vec!(0x00)), args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[0x1F] = Some(Opcode{ code: 0x1F, fmt: "JZ",
                         types: Some(vec!(0x00)), args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[0x20] = Some(Opcode{ code: 0x20, fmt: "RETN",
                         types: Some(vec!(0x00)), args: None });
  x[0x21] = Some(Opcode{ code: 0x21, fmt: "DESTRUCT",
                         types: Some(vec!(0x01)),
                         args: Some(hashmap!(0x01 => vec!(Size(2), Offset(2), Size(2)))) });
  x[0x22] = Some(Opcode{ code: 0x22, fmt: "NOTI",
                         types: Some(vec!(0x03)), args: None });
  x[0x23] = Some(Opcode{ code: 0x23, fmt: "DECISP",
                         types: Some(vec!(0x03)), args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[0x24] = Some(Opcode{ code: 0x24, fmt: "INCISP",
                         types: Some(vec!(0x03)), args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[0x25] = Some(Opcode{ code: 0x25, fmt: "JNZ",
                         types: Some(vec!(0x00)), args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[0x26] = Some(Opcode{ code: 0x26, fmt: "CPDOWNBP",
                         types: Some(vec!(0x01)),
                         args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[0x27] = Some(Opcode{ code: 0x27, fmt: "CPTOPBP",
                         types: Some(vec!(0x01)),
                         args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[0x28] = Some(Opcode{ code: 0x28, fmt: "DECIBP",
                         types: Some(vec!(0x03)), args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[0x29] = Some(Opcode{ code: 0x29, fmt: "INCIBP",
                         types: Some(vec!(0x03)), args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[0x2A] = Some(Opcode{ code: 0x2A, fmt: "SAVEBP",
                         types: Some(vec!(0x00)), args: None });
  x[0x2B] = Some(Opcode{ code: 0x2B, fmt: "RESTOREBP",
                         types: Some(vec!(0x00)), args: None });
  x[0x2C] = Some(Opcode{ code: 0x2C, fmt: "STORE_STATE",
                         types: Some(vec!(0x10)),
                         args: Some(hashmap!(0x10 => vec!(Size(4), Size(4)))) });
  x[0x2D] = Some(Opcode{ code: 0x2D, fmt: "NOP",
                         types: Some(vec!(0x00)), args: None });
  x[0x42] = Some(Opcode{ code: 0x42, fmt: "T",
                         types: None,
                         args: Some(hashmap!(0x00 => vec!(Size(4)))) }); // hack

  // Does this actually need to be boxed???
  x.into_boxed_slice()
}

// TODO options for engine types for NWN/DA/DA2
pub fn get_nwtypes() -> Box<[Option<NWType>]> {

  let mut x:Vec<Option<NWType>> = repeat(true).take(MAX_STYPES).map(|_| None).collect();

  // Unknown types
  x[0x00] = Some(NWType{ code: 0x00, abbr: None, desc: "Null?" });
  x[0x01] = Some(NWType{ code: 0x01, abbr: None, desc: "Copy?" });

  // unary types
  x[0x03] = Some(NWType{ code: 0x03, abbr: Some("I"), desc: "Integer" });
  x[0x04] = Some(NWType{ code: 0x04, abbr: Some("F"), desc: "Float" });
  x[0x05] = Some(NWType{ code: 0x05, abbr: Some("S"), desc: "String" });
  x[0x06] = Some(NWType{ code: 0x06, abbr: Some("O"), desc: "Object" });

  // engine types 0x10 - 0x1F
  x[0x10] = Some(NWType{ code: 0x10, abbr: None, desc: "Effect" });
  x[0x11] = Some(NWType{ code: 0x11, abbr: None, desc: "Event" });
  x[0x12] = Some(NWType{ code: 0x12, abbr: None, desc: "Location" });
  x[0x13] = Some(NWType{ code: 0x13, abbr: None, desc: "Talent" });

  // binary types
  x[0x20] = Some(NWType{ code: 0x20, abbr: Some("II"), desc: "Integer, Integer" });
  x[0x21] = Some(NWType{ code: 0x21, abbr: Some("FF"), desc: "Float, Float" });
  x[0x22] = Some(NWType{ code: 0x22, abbr: Some("OO"), desc: "Object, Object" });
  x[0x23] = Some(NWType{ code: 0x23, abbr: Some("SS"), desc: "String, String" });
  x[0x24] = Some(NWType{ code: 0x24, abbr: Some("TT"), desc: "Structure, Structure" });
  x[0x25] = Some(NWType{ code: 0x25, abbr: Some("IF"), desc: "Integer, Float" });
  x[0x26] = Some(NWType{ code: 0x26, abbr: Some("FI"), desc: "Float, Integer" });

  // engine types 0x30 - 0x39
  x[0x30] = Some(NWType{ code: 0x30, abbr: None, desc: "Effect, Effect" });
  x[0x31] = Some(NWType{ code: 0x31, abbr: None, desc: "Event, Event" });
  x[0x32] = Some(NWType{ code: 0x32, abbr: None, desc: "Location, Location" });
  x[0x33] = Some(NWType{ code: 0x33, abbr: None, desc: "Talent, Talent" });

  // undocumented, but are legal args for EQUALxx?
  x[0x34] = Some(NWType{ code: 0x34, abbr: None, desc: "???" });
  x[0x35] = Some(NWType{ code: 0x35, abbr: None, desc: "???" });
  x[0x36] = Some(NWType{ code: 0x36, abbr: None, desc: "???" });
  x[0x37] = Some(NWType{ code: 0x37, abbr: None, desc: "???" });
  x[0x38] = Some(NWType{ code: 0x38, abbr: None, desc: "???" });
  x[0x39] = Some(NWType{ code: 0x39, abbr: None, desc: "???" });
  // end engine types

  x[0x3A] = Some(NWType{ code: 0x3A, abbr: Some("VV"), desc: "Vector, Vector" });
  x[0x3B] = Some(NWType{ code: 0x3B, abbr: Some("VF"), desc: "Vector, Float" });
  x[0x3C] = Some(NWType{ code: 0x3c, abbr: Some("FV"), desc: "Float, Vector" });

  x.into_boxed_slice()
}
