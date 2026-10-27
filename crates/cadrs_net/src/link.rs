//! The link a document is shared by: which document, the room it is shared in, and how to
//! reach its owner.
//!
//! Like a bevy_iroh ticket, a link is a **capability**: whoever holds it knows the room, and
//! the owner lets them in with the link's permission (when link sharing is on). Turning link
//! sharing off makes every link to the document useless.

use std::{fmt, str::FromStr};

use bevy_iroh::iroh::EndpointAddr;
use cadrs_core::DocumentId;
use serde::{Deserialize, Serialize};

/// What a link starts with, so a mistyped or truncated paste fails with a sentence.
const PREFIX: &str = "cadrs";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ShareLink {
    pub document: DocumentId,
    /// The document's name when the link was made, to show before anything has arrived.
    pub name: String,
    /// The room's topic ([`cadrs_core::collab::Sharing::topic`]).
    pub topic: [u8; 32],
    /// The owner, and where they could be reached when the link was made (iroh finds them by
    /// their key when that has changed).
    pub owner: EndpointAddr,
}

/// Lower case base32, because links get pasted into chat clients that like to capitalise.
impl fmt::Display for ShareLink {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let bytes = postcard::to_stdvec(self).map_err(|_| fmt::Error)?;
        let mut text = data_encoding::BASE32_NOPAD.encode(&bytes);
        text.make_ascii_lowercase();
        write!(f, "{PREFIX}{text}")
    }
}

impl FromStr for ShareLink {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, String> {
        let s = s.trim();
        let body = s
            .get(..PREFIX.len())
            .filter(|head| head.eq_ignore_ascii_case(PREFIX))
            .map(|_| &s[PREFIX.len()..])
            .ok_or("A cadrs link starts with \"cadrs\"")?;
        if body.is_empty() {
            return Err("The link is empty".into());
        }
        let bytes = data_encoding::BASE32_NOPAD
            .decode(body.to_ascii_uppercase().as_bytes())
            .map_err(|_| "The link is not complete")?;
        postcard::from_bytes(&bytes).map_err(|_| "The link is not complete".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bevy_iroh::iroh::SecretKey;

    fn link() -> ShareLink {
        ShareLink {
            document: DocumentId::new(),
            name: "Bracket".into(),
            topic: [7; 32],
            owner: EndpointAddr::new(SecretKey::generate().public()),
        }
    }

    #[test]
    fn round_trips_through_a_chat_client() {
        let l = link();
        let mangled = format!("  {}  ", l.to_string().to_uppercase());
        assert_eq!(mangled.parse::<ShareLink>().unwrap(), l);
    }

    #[test]
    fn bad_input_says_what_is_wrong() {
        assert!("".parse::<ShareLink>().is_err());
        assert!("cadrs".parse::<ShareLink>().is_err());
        assert!("chanabc".parse::<ShareLink>().is_err());
        let text = link().to_string();
        assert!(text[..20].parse::<ShareLink>().is_err());
    }
}
