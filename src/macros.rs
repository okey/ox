#![macro_use]

macro_rules! step_or_return {
  ($t:ident, $n:expr, $lim:expr) => (
    if ($t.1 + $n) <= $lim { ($t.1, $t.1 + $n) } else {
      return Err(DecodeError{
        message: format!("step failed at {:?}+{} < {}", $t, $n, $lim), byte: 0})
    });
}

macro_rules! step_or_return2 {
  ($t:ident, $n:expr, $lim:expr) => (
    if ($t.1 + $n) <= $lim { ($t.1, $t.1 + $n) } else {
      return Err(CommandStreamError(DecodeError{
        message: format!("step failed at {:?}+{} < {}", $t, $n, $lim), byte: 0}))
    });
}

macro_rules! println_err(
  ($($arg:tt)*) => (
    match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
      Ok(_) => {},
      Err(x) => panic!("Unable to write to stderr: {}", x),
    }));
