use crate::error::protocol_error::ProtocolError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestType {
    Produce,
    Fetch,
    CreateTopic,
    CommitOffset,
    ListTopics,
}

impl RequestType {
    pub fn code(self) -> u16 {
        match self {
            RequestType::Produce => 1,
            RequestType::Fetch => 2,
            RequestType::CreateTopic => 3,
            RequestType::CommitOffset => 4,
            RequestType::ListTopics => 5,
        }
    }

    pub fn from_code(code: u16) -> Result<Self, ProtocolError> {
        match code {
            1 => Ok(RequestType::Produce),
            2 => Ok(RequestType::Fetch),
            3 => Ok(RequestType::CreateTopic),
            4 => Ok(RequestType::CommitOffset),
            5 => Ok(RequestType::ListTopics),
            _ => Err(ProtocolError::new(format!(
                "Unknown request type code: {code}"
            ))),
        }
    }
}
