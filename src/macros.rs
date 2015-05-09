#![macro_use]

macro_rules! println_err(
  ($($arg:tt)*) => (
    match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
      Ok(_) => {},
      Err(x) => panic!("Unable to write to stderr: {}", x),
    }));

macro_rules! read_exact {
  ($rdr:ident, $arr:expr, $n:expr, $b:expr) => {
    {
      let _sz = try!($rdr.read($arr));
      if _sz != $n {
        // TODO accept byte count
        op_err!($b + _sz, "Unexpected EOF: got {} bytes but expected {}", _sz, $n)
      }
      _sz
    }
  }
}

macro_rules! output {
  ($wtr:ident, $fmt:expr, $($arg:expr),*) =>
    (try!($wtr.write(format!($fmt, $($arg),*).as_bytes())));
}

macro_rules! op_err {
  ($byte:expr, $fmt:expr) =>
    (return Err(OpStreamError($fmt.to_string(), $byte)));
  ($byte:expr, $fmt:expr, $($arg:expr),*) =>
    (return Err(OpStreamError(format!($fmt, $($arg),*).to_string(), $byte)))
}

macro_rules! data_err {
  ($fmt:expr, $($arg:expr),*) =>
    (return Err(DataError(format!($fmt, $($arg),*).to_string())))
}
