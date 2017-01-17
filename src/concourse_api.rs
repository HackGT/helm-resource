extern crate serde;
extern crate serde_json;

use self::serde::{
    Deserialize,
    Serialize,
};
use self::serde_json::error::Result as JsonResult;
use std::io::{
    self,
};

#[derive(Deserialize)]
pub struct Source {
    pub url: String,
    pub username: String,
    pub password: String,
    pub namespace: String,
    pub skip_tls_verify: Option<bool>,
    pub ca_data: Option<String>,
}

#[derive(Serialize, Deserialize)]
pub struct Version {
    pub digest: String,
}

#[derive(Deserialize)]
pub struct CheckRequest {
    pub source: Source,
    pub version: Option<Version>,
}

pub type InRequest = CheckRequest;

#[derive(Serialize)]
pub struct InResponse<M>
where M: Serialize
{
    pub version: Version,
    pub metadata: M,
}

pub fn receive_message<T>() -> JsonResult<T>
where T: Deserialize
{
    let mut buffer = String::new();
    try!(io::stdin().read_line(&mut buffer));
    serde_json::from_str::<T>(&buffer)
}

pub fn send_message<T>(message: &T) -> JsonResult<()>
where T: Serialize
{
    let message_txt = try!(serde_json::to_string::<T>(message));
    println!("{}", message_txt);
    Ok(())
}
