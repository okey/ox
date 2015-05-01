#![macro_use]

macro_rules! step_or_return {
  ($t:ident, $n:expr, $lim:expr, $err:expr) => (
    if ($t.1 + $n) < $lim { ($t.1, $t.1 + $n) } else { return $err }
    )
}

macro_rules! println_err(
  ($($arg:tt)*) => (
    match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
      Ok(_) => {},
      Err(x) => panic!("Unable to write to stderr: {}", x),
    }));
