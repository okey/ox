use std::iter::repeat;
use std::collections::{HashMap,HashSet};
use std::fmt;
use self::Operand::*;


#[derive(Debug)]
#[derive(PartialEq)]
#[derive(Clone,Copy)] // blergh
#[allow(non_camel_case_types)]
pub enum OpcodeE {
  CPDOWNSP = 0x01,
  RSADD = 0x02,
  CPTOPSP = 0x03,
  CONST = 0x04,
  ACTION = 0x05,
  LOGANDII = 0x06,
  LOGORII = 0x07,
  INCORII = 0x08,
  EXCORII = 0x09,
  BOOLANDII = 0x0A,
  EQUAL = 0x0B,
  NEQUAL = 0x0C,
  GEQ = 0x0D,
  GT = 0x0E,
  LT = 0x0F,
  LEQ = 0x10,
  SHLEFTII = 0x11,
  SHRIGHTII = 0x12,
  USHRIGHTII = 0x13,
  ADD = 0x14,
  SUB = 0x15,
  MUL = 0x16,
  DIV = 0x17,
  MODII = 0x18,
  NEG = 0x19,
  COMPI = 0x1A,
  MOVSP = 0x1B,
  STORE_STATEALL = 0x1C,
  JMP = 0x1D,
  JSR = 0x1E,
  JZ = 0x1F,
  RETN = 0x20,
  DESTRUCT = 0x21,
  NOTI = 0x22,
  DECISP = 0x23,
  INCISP = 0x24,
  JNZ = 0x25,
  CPDOWNBP = 0x26,
  CPTOPBP = 0x27,
  DECIBP = 0x28,
  INCIBP = 0x29,
  SAVEBP = 0x2A,
  RESTOREBP = 0x2B,
  STORE_STATE = 0x2C,
  NOP = 0x2D,
  CP_x37_DA2_QQ = 0x37,
  T = 0x42,
}

impl fmt::UpperHex for OpcodeE {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "{:X}", *self as usize)
  }
}

impl fmt::Display for OpcodeE {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    fmt::Debug::fmt(self, f)
  }
}

#[derive(Debug)]
pub struct Opcode {
  pub code: OpcodeE,
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
  ArgCount(usize),
}

pub struct OpPayload<'a > {
  pub bytes_read: usize,
  pub op: &'a Opcode,
  pub _type: Option<u8>,
  pub args: Vec<(&'a Operand, Vec<u8>)>
} // find some way to implement Show with an instance payload type?

#[derive(Debug)]
#[derive(Clone,Copy)]
pub enum NWTypeE {
  // unknown types
  NULLQ = 0x00,
  COPYQ = 0x01,

  // unary types
  I = 0x03,
  F = 0x04,
  S = 0x05,
  O = 0x06,

  // engine types 0x10 - 0x1F
  Effect = 0x10,
  Event = 0x11,
  Location = 0x12,
  Talent = 0x13,

  // binary types
  II = 0x20,
  FF = 0x21,
  OO = 0x22,
  SS = 0x23,
  TT = 0x24,
  IF = 0x25,
  FI = 0x26,

  // engine types 0x30 - 0x39
  EffectEffect = 0x30,
  EventEvent = 0x31,
  LocationLocation = 0x32,
  TalentTalent = 0x33,

  // undocumented, but are legal args for EQUALxx?
  Unknownx34 = 0x34,
  Unknownx35 = 0x35,
  Unknownx36 = 0x36,
  Unknownx37 = 0x37,
  Unknownx38 = 0x38,
  Unknownx39 = 0x39,
  // end engine types

  // vector types
  VV = 0x3A,
  VF = 0x3B,
  FV = 0x3C
}

pub struct NWType {
  pub code: NWTypeE,
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


// could have a map with a tuple key type: (OpcodeE, NWTypeE) => vec!(args) ???
// still doesn't prevent accidentally trampling mappings etc, though?

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

/*const array = {
    let mut array = [0; 1024];

    for (i, element) in array.iter_mut().enumerate().take(2) {
        *element = (i + 7);
    }

    array
};*/


pub fn get_opcodes() -> Box<[Option<Opcode>]> {
  use self::OpcodeE::*;
  // Take N * None without Clone or Copy
  let mut x:Vec<Option<Opcode>> = repeat(true).take(MAX_OPCODES).map(|_| None).collect();
  // TODO fix python generation of codes for new format
  // TODO auto insertion of generated values?
  // TODO automatically check for duplicate keys
  x[CPDOWNSP as usize] = Some(Opcode{ code: CPDOWNSP, types: Some(vec!(0x01)),
                                      args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[RSADD as usize] = Some(Opcode{ code: RSADD,
                                   types: Some(vec!(0x03, 0x04, 0x05, 0x06, 0x13)), args: None });
  x[CPTOPSP as usize] = Some(Opcode{ code: CPTOPSP, types: Some(vec!(0x01)),
                                     args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[CONST as usize] = Some(Opcode{ code: CONST, types: Some(vec!(0x03, 0x04, 0x05, 0x06)),
                                   args: Some(hashmap!(0x03 => vec!(Integer(4)),
                                                       0x04 => vec!(Float(4)),
                                                       0x05 => vec!(Size(2), String),
                                                       0x06 => vec!(Object(4)))) });
  x[ACTION as usize] = Some(Opcode{ code: ACTION, types: Some(vec!(0x00)),
                                    args: Some(hashmap!(0x00 => vec!(Routine(2), ArgCount(1)))) });
  x[LOGANDII as usize] = Some(Opcode{ code: LOGANDII, types: Some(vec!(0x20)), args: None });
  x[LOGORII as usize] = Some(Opcode{ code: LOGORII, types: Some(vec!(0x20)), args: None });
  x[INCORII as usize] = Some(Opcode{ code: INCORII, types: Some(vec!(0x20)), args: None });
  x[EXCORII as usize] = Some(Opcode{ code: EXCORII, types: Some(vec!(0x20)), args: None });
  x[BOOLANDII as usize] = Some(Opcode{ code: BOOLANDII, types: Some(vec!(0x20)), args: None });
  x[EQUAL as usize] = Some(Opcode{ code: EQUAL,
                                   types: Some(vec!(0x20, 0x21, 0x22, 0x23, 0x024, 0x30, 0x31,
                                                    0x32, 0x33, 0x34, 0x35, 0x36, 0x37, 0x38,
                                                    0x39)),
                                   args: Some(hashmap!(0x20 => vec!(), 0x21 => vec!(),
                                                       0x22 => vec!(), 0x23 => vec!(),
                                                       0x24 => vec!(), 0x30 => vec!(),
                                                       0x31 => vec!(), 0x32 => vec!(),
                                                       0x33 => vec!(), 0x34 => vec!(),
                                                       0x35 => vec!(), 0x36 => vec!(),
                                                       0x37 => vec!(), 0x38 => vec!(),
                                                       0x39 => vec!(), 0x24 => vec!(Size(2)))) });
  x[NEQUAL as usize] = Some(Opcode{ code: NEQUAL,
                                    types: Some(vec!(0x20, 0x21, 0x22, 0x23, 0x30, 0x31, 0x32,
                                                     0x33, 0x34, 0x35, 0x36, 0x37, 0x38, 0x39)),
                                    args: Some(hashmap!(0x20 => vec!(), 0x21 => vec!(),
                                                        0x22 => vec!(), 0x23 => vec!(),
                                                        0x24 => vec!(), 0x30 => vec!(),
                                                        0x31 => vec!(), 0x32 => vec!(),
                                                        0x33 => vec!(), 0x34 => vec!(),
                                                        0x35 => vec!(), 0x36 => vec!(),
                                                        0x37 => vec!(), 0x38 => vec!(),
                                                        0x39 => vec!(), 0x24 => vec!(Size(2)))) });
  x[GEQ as usize] = Some(Opcode{ code: GEQ, types: Some(vec!(0x20, 0x21)), args: None });
  x[GT as usize] = Some(Opcode{ code: GT, types: Some(vec!(0x20, 0x21)), args: None });
  x[LT as usize] = Some(Opcode{ code: LT, types: Some(vec!(0x20, 0x21)), args: None });
  x[LEQ as usize] = Some(Opcode{ code: LEQ, types: Some(vec!(0x20, 0x21)), args: None });
  x[SHLEFTII as usize] = Some(Opcode{ code: SHLEFTII, types: Some(vec!(0x20)), args: None });
  x[SHRIGHTII as usize] = Some(Opcode{ code: SHRIGHTII, types: Some(vec!(0x20)), args: None });
  x[USHRIGHTII as usize] = Some(Opcode{ code: USHRIGHTII, types: Some(vec!(0x20)), args: None });
  x[ADD as usize] = Some(Opcode{ code: ADD, types: Some(vec!(0x20, 0x25, 0x26, 0x21, 0x23, 0x3a)),
                                 args: None });
  x[SUB as usize] = Some(Opcode{ code: SUB,
                                 types: Some(vec!(0x20, 0x25, 0x26, 0x21, 0x3a)), args: None });
  x[MUL as usize] = Some(Opcode{ code: MUL, types: Some(vec!(0x20, 0x25, 0x26, 0x21, 0x3b, 0x3c)),
                                 args: None });
  x[DIV as usize] = Some(Opcode{ code: DIV,
                                 types: Some(vec!(0x20, 0x25, 0x26, 0x21, 0x3b)), args: None });
  x[MODII as usize] = Some(Opcode{ code: MODII, types: Some(vec!(0x20)), args: None });
  x[NEG as usize] = Some(Opcode{ code: NEG, types: Some(vec!(0x03, 0x04)), args: None });
  x[COMPI as usize] = Some(Opcode{ code: COMPI, types: Some(vec!(0x03)), args: None });
  x[MOVSP as usize] = Some(Opcode{ code: MOVSP, types: Some(vec!(0x00)),
                                   args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[STORE_STATEALL as usize] = Some(Opcode{ code: STORE_STATEALL,
                                            types: Some(vec!(0x08)), args: None });
  x[JMP as usize] = Some(Opcode{ code: JMP, types: Some(vec!(0x00)),
                                 args: Some(hashmap!(0x00 => vec!(Offset(4)))) });;
  x[JSR as usize] = Some(Opcode{ code: JSR, types: Some(vec!(0x00)),
                                 args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[JZ as usize] = Some(Opcode{ code: JZ, types: Some(vec!(0x00)),
                                args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[RETN as usize] = Some(Opcode{ code: RETN, types: Some(vec!(0x00)), args: None });
  x[DESTRUCT as usize] = Some(Opcode{ code: DESTRUCT, types: Some(vec!(0x01)),
                                      args: Some(hashmap!(0x01 =>
                                                          vec!(Size(2), Offset(2), Size(2)))) });
  x[NOTI as usize] = Some(Opcode{ code: NOTI, types: Some(vec!(0x03)), args: None });
  x[DECISP as usize] = Some(Opcode{ code: DECISP, types: Some(vec!(0x03)),
                                    args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[INCISP as usize] = Some(Opcode{ code: INCISP, types: Some(vec!(0x03)),
                                    args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[JNZ as usize] = Some(Opcode{ code: JNZ, types: Some(vec!(0x00)),
                                 args: Some(hashmap!(0x00 => vec!(Offset(4)))) });
  x[CPDOWNBP as usize] = Some(Opcode{ code: CPDOWNBP, types: Some(vec!(0x01)),
                                      args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[CPTOPBP as usize] = Some(Opcode{ code: CPTOPBP, types: Some(vec!(0x01)),
                                     args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2)))) });
  x[DECIBP as usize] = Some(Opcode{ code: DECIBP, types: Some(vec!(0x03)),
                                    args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[INCIBP as usize] = Some(Opcode{ code: INCIBP, types: Some(vec!(0x03)),
                                    args: Some(hashmap!(0x03 => vec!(Offset(4)))) });
  x[SAVEBP as usize] = Some(Opcode{ code: SAVEBP, types: Some(vec!(0x00)), args: None });
  x[RESTOREBP as usize] = Some(Opcode{ code: RESTOREBP, types: Some(vec!(0x00)), args: None });
  x[STORE_STATE as usize] = Some(Opcode{ code: STORE_STATE, types: Some(vec!(0x10)),
                                         args: Some(hashmap!(0x10 => vec!(Size(4), Size(4)))) });
  x[NOP as usize] = Some(Opcode{ code: NOP, types: Some(vec!(0x00)), args: None });
  x[CP_x37_DA2_QQ as usize] = Some(Opcode{ code: CP_x37_DA2_QQ, // PROBABLY PARTIALLY INCORRECT!
                                           types: Some(vec!(0x01)),
                                           args: Some(hashmap!(0x01 => vec!(Offset(4), Size(2))))}); // MADE UP
  x[T as usize] = Some(Opcode{ code: T, types: None,
                               args: Some(hashmap!(0x00 => vec!(Size(4)))) }); // hack

  // Does this actually need to be boxed???
  x.into_boxed_slice()
}

// TODO options for engine types for NWN/DA/DA2 (how?)
pub fn get_nwtypes() -> Box<[Option<NWType>]> {

  use self::NWTypeE::*;
  let mut x: Vec<Option<NWType>> = repeat(true).take(MAX_STYPES).map(|_| None).collect();

  // Unknown types
  x[NULLQ as usize] = Some(NWType{ code: NULLQ, abbr: None, desc: "Null?" });
  x[COPYQ as usize] = Some(NWType{ code: COPYQ, abbr: None, desc: "Copy?" });

  // unary types
  x[I as usize] = Some(NWType{ code: I, abbr: Some("I"), desc: "Integer" });
  x[F as usize] = Some(NWType{ code: F, abbr: Some("F"), desc: "Float" });
  x[S as usize] = Some(NWType{ code: S, abbr: Some("S"), desc: "String" });
  x[O as usize] = Some(NWType{ code: O, abbr: Some("O"), desc: "Object" });

  // engine types 0x10 - 0x1F
  x[Effect as usize] = Some(NWType{ code: Effect, abbr: None, desc: "Effect" });
  x[Event as usize] = Some(NWType{ code: Event, abbr: None, desc: "Event" });
  x[Location as usize] = Some(NWType{ code: Location, abbr: None, desc: "Location" });
  x[Talent as usize] = Some(NWType{ code: Talent, abbr: None, desc: "Talent" });

  // binary types
  x[II as usize] = Some(NWType{ code: II, abbr: Some("II"), desc: "Integer, Integer" });
  x[FF as usize] = Some(NWType{ code: FF, abbr: Some("FF"), desc: "Float, Float" });
  x[OO as usize] = Some(NWType{ code: OO, abbr: Some("OO"), desc: "Object, Object" });
  x[SS as usize] = Some(NWType{ code: SS, abbr: Some("SS"), desc: "String, String" });
  x[TT as usize] = Some(NWType{ code: TT, abbr: Some("TT"), desc: "Structure, Structure" });
  x[IF as usize] = Some(NWType{ code: IF, abbr: Some("IF"), desc: "Integer, Float" });
  x[FI as usize] = Some(NWType{ code: FI, abbr: Some("FI"), desc: "Float, Integer" });

  // engine types 0x30 - 0x39
  x[EffectEffect as usize] = Some(NWType{ code: EffectEffect, abbr: None,
                                          desc: "Effect, Effect" });
  x[EventEvent as usize] = Some(NWType{ code: EventEvent, abbr: None, desc: "Event, Event" });
  x[LocationLocation as usize] = Some(NWType{ code: LocationLocation, abbr: None,
                                     desc: "Location, Location" });
  x[TalentTalent as usize] = Some(NWType{ code: TalentTalent, abbr: None,
                                          desc: "Talent, Talent" });

  // undocumented, but are legal args for EQUALxx?
  x[Unknownx34 as usize] = Some(NWType{ code: Unknownx34, abbr: None, desc: "???" });
  x[Unknownx35 as usize] = Some(NWType{ code: Unknownx35, abbr: None, desc: "???" });
  x[Unknownx36 as usize] = Some(NWType{ code: Unknownx36, abbr: None, desc: "???" });
  x[Unknownx37 as usize] = Some(NWType{ code: Unknownx37, abbr: None, desc: "???" });
  x[Unknownx38 as usize] = Some(NWType{ code: Unknownx38, abbr: None, desc: "???" });
  x[Unknownx39 as usize] = Some(NWType{ code: Unknownx39, abbr: None, desc: "???" });
  // end engine types

  // vector types
  x[VV as usize] = Some(NWType{ code: VV, abbr: Some("VV"), desc: "Vector, Vector" });
  x[VF as usize] = Some(NWType{ code: VF, abbr: Some("VF"), desc: "Vector, Float" });
  x[FV as usize] = Some(NWType{ code: FV, abbr: Some("FV"), desc: "Float, Vector" });

  x.into_boxed_slice()
}
