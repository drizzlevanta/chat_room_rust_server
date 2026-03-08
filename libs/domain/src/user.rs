use chrono::{DateTime, Utc};
use std::{fmt, str::FromStr};
use thiserror::Error;
use uuid::Uuid;

/// Domain representation of a user
#[derive(Clone)]
pub struct User {
    pub id: Uuid, // Unique identifier for the user. This is the public id.
    pub name: String,
    pub status: Option<Status>,
    pub room: Option<Uuid>, // The room the user is currently in. This is the public id
    pub last_seen: Option<DateTime<Utc>>, // Timestamp of the last activity
}

/// Possible statuses for a user
#[derive(Copy, Clone, PartialEq, Eq, Debug)]
pub enum Status {
    Online,
    Offline,
    Away,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Status::Online => "online",
            Status::Offline => "offline",
            Status::Away => "away",
        };
        write!(f, "{s}")
    }
}

/// Converting db strings to domain enum.
impl FromStr for Status {
    type Err = ParseUserStatusError;

    fn from_str(input: &str) -> Result<Status, Self::Err> {
        match input {
            "online" => Ok(Status::Online),
            "offline" => Ok(Status::Offline),
            "away" => Ok(Status::Away),
            _ => Err(ParseUserStatusError::InvalidStatus(input.to_string())),
        }
    }
}

#[derive(Debug, Error, Clone)]
pub enum ParseUserStatusError {
    #[error("invalid user status: {0}")]
    InvalidStatus(String),
}
