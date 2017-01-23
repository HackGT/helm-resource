extern crate serde_json;

use std::fmt;
use std::error::Error;
use self::serde_json::Map;
use self::serde_json::Value;
use super::url::ParseError;
pub use super::rustache::RustacheError;
pub use super::curl::Error as CurlError;
pub use std::io::Error as IoError;

#[derive(Debug)]
pub enum HelmError {
    Io(IoError),
    FailedToCreateKubeConfig(RustacheError),
    Net(CurlError),
    CmdFailed(String),
    UrlParse(ParseError),
    NoCaData,
    WrongKubeApiFormat(Map<String, Value>),
}

impl fmt::Display for HelmError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &HelmError::CmdFailed(ref cmd) =>
                f.write_fmt(format_args!("could not run command `{}`", cmd)),
            &HelmError::WrongKubeApiFormat(ref object) =>
                f.write_fmt(format_args!("could not parse api `{:?}`", object)),
            _ => write!(f, "{}", self.description()),
        }
    }
}

impl Error for HelmError {
    fn description(&self) -> &str {
        match (self, self.cause()) {
            (_, Some(e)) => e.description(),
            (&HelmError::Io(_), None) => unreachable!(),
            (&HelmError::Net(_), None) => unreachable!(),
            (&HelmError::UrlParse(_), None) => unreachable!(),
            (&HelmError::FailedToCreateKubeConfig(_), _) => "rustache templating error",
            (&HelmError::CmdFailed(ref cmd), _) => cmd,
            (&HelmError::WrongKubeApiFormat(_), _) => "could not parse k8s api",
            (&HelmError::NoCaData, _) => "No ca data given and skip_tls_verify = false",
        }
    }

    fn cause(&self) -> Option<&::std::error::Error> {
        match *self {
            HelmError::Io(ref e) => Some(e),
            HelmError::Net(ref e) => Some(e),
            HelmError::UrlParse(ref e) => Some(e),
            _ => None,
        }
    }
}

impl From<IoError> for HelmError {
    fn from(e: IoError) -> Self {
        HelmError::Io(e)
    }
}

impl From<RustacheError> for HelmError {
    fn from(e: RustacheError) -> Self {
        HelmError::FailedToCreateKubeConfig(e)
    }
}

impl From<CurlError> for HelmError {
    fn from(e: CurlError) -> Self {
        HelmError::Net(e)
    }
}

impl From<ParseError> for HelmError {
    fn from(e: ParseError) -> Self {
        HelmError::UrlParse(e)
    }
}
