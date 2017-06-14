
#[macro_export]
macro_rules! format_err {
    ($s:expr) => {
        Error::from($s)
    };
    ($s:expr, $($arg:expr),*) => {
        Error::from(format!($s, $($arg),*))
    };
}


#[macro_export]
macro_rules! bail {
    ($s:expr) => {
        return Err(format_err!($s));
    };
    ($s:expr, $($arg:expr),*) => {
        return Err(format_err!($s, $($arg),*));
    };
}


pub type Result<T> = ::std::result::Result<T, Error>;


pub enum Error {
    Msg(String),
    Io(::std::io::Error),
}

use ::std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use Error::*;
        match *self {
            Msg(ref s)  => write!(f, "Msg: {}", s),
            Io(ref e)   => write!(f, "Io: {}", e),
        }
    }
}

impl From<::std::io::Error> for Error {
    fn from(e: ::std::io::Error) -> Error {
        Error::Io(e)
    }
}

impl From<String> for Error {
    fn from(s: String) -> Error {
        Error::Msg(s)
    }
}

impl<'a> From<&'a str> for Error {
    fn from(s: &'a str) -> Error {
        Error::Msg(String::from(s))
    }
}
