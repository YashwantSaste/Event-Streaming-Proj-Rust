use crate::error::protocol_error::ProtocolError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseStatus {
    Ok,
    Error,
}

impl ResponseStatus {
    pub fn code(self) -> u16 {
        match self {
            ResponseStatus::Ok => 0,
            ResponseStatus::Error => 1,
        }
    }

    pub fn from_code(code: u16) -> Result<Self, ProtocolError> {
        match code {
            0 => Ok(ResponseStatus::Ok),
            1 => Ok(ResponseStatus::Error),
            _ => Err(ProtocolError::new(format!(
                "Unknown response status code: {code}"
            ))),
        }
    }
}
