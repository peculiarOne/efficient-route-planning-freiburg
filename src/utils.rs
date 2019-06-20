use std::borrow::Cow;
use std::string::FromUtf8Error;

pub fn bytes_to_string(bytes: Cow<'_, [u8]>) -> Result<String, FromUtf8Error> {
    String::from_utf8(bytes.to_vec())
}
