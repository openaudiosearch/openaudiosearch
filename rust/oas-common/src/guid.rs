use std::fmt;
use std::ops::Deref;
use std::{ops::Range, str::FromStr};

use serde::de::Visitor;
use serde::{Deserialize, Serialize};

use crate::util::id_from_uuid;

pub const SEPERATOR: &str = "_";

#[derive(Debug, Default, PartialEq, Clone)]
pub struct GuidParseError;
impl std::error::Error for GuidParseError {}
impl fmt::Display for GuidParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Failed to parse GUID")
    }
}

/// A GUID is a string that is composed of a typ string and a id string.
#[derive(Debug, Clone, PartialEq)]
pub struct Guid {
    guid: String,
    typ: Range<usize>,
    id: Range<usize>,
}

impl fmt::Display for Guid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.guid)
    }
}

impl Serialize for Guid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.guid)
    }
}

impl<'de> Deserialize<'de> for Guid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct GuidVisitor;
        impl<'de> Visitor<'de> for GuidVisitor {
            type Value = Guid;
            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid GUID string in the form \"typ_id\"")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Guid::from_str(&v).map_err(|e| E::custom(format!("{}", e)))
            }
        }
        deserializer.deserialize_str(GuidVisitor)
    }
}

impl Guid {
    /// Get the GUID string
    pub fn guid(&self) -> &str {
        &self.guid
    }

    /// Turn into a GUID string
    pub fn into_guid(self) -> String {
        self.guid
    }

    // pub fn from_string_unchecked(s: String) -> Self {
    //     Self::from_str(&s).unwrap()
    // }

    /// Get the type part of the GUID
    pub fn typ(&self) -> &str {
        &self.guid[self.typ.start..self.typ.end]
    }

    /// Get the id part of the GUID
    pub fn id(&self) -> &str {
        &self.guid[self.id.start..self.id.end]
    }

    pub fn from_parts(typ: &str, id: &str) -> Self {
        let guid = format!("{}{}{}", typ, SEPERATOR, id);
        Self {
            typ: 0..typ.len(),
            id: (typ.len() + SEPERATOR.len())..guid.len(),
            guid,
        }
    }

    pub fn typ_and_random(typ: &str) -> Self {
        Self::from_parts(typ, &id_from_uuid())
    }
}

impl Deref for Guid {
    type Target = str;
    fn deref(&self) -> &Self::Target {
        &self.guid
    }
}

impl FromStr for Guid {
    type Err = GuidParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split_terminator(SEPERATOR).collect();
        if parts.len() == 2 {
            let typ = parts[0];
            let id = parts[1];
            if s.len() != typ.len() + id.len() + SEPERATOR.len() {
                Err(GuidParseError)
            } else if id.is_empty() || typ.is_empty() {
                Err(GuidParseError)
            } else {
                Ok(Self {
                    guid: s.to_string(),
                    typ: 0..typ.len(),
                    id: (typ.len() + SEPERATOR.len())..s.len(),
                })
            }
        } else {
            Err(GuidParseError)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn guid() {
        let guid = Guid::from_parts("foo", "bar");
        assert_eq!(guid.guid(), "foo_bar");
        assert_eq!(&*guid, "foo_bar");
        assert_eq!(guid.typ(), "foo");
        assert_eq!(guid.id(), "bar");
    }
    #[test]
    fn parse_guid() {
        let s = "media_pic1312";
        let guid = Guid::from_str(&s).expect("failed to parse uid");
        assert_eq!(guid.typ(), "media");
        assert_eq!(guid.id(), "pic1312");
        let invalid = [
            "_",
            "__",
            "_foo",
            "foo_",
            "foo_bar_",
            "foo_bar_boo",
            "_foo_bar",
            "foo_bar_",
        ];
        for s in invalid.iter() {
            assert_eq!(Guid::from_str(&s).err(), Some(GuidParseError),);
        }
    }

    #[test]
    fn serde() {
        use serde::{Deserialize, Serialize};
        let src = r#"{ "guid": "foo_bar" }"#;
        #[derive(Serialize, Deserialize, Debug)]
        struct Data {
            guid: Guid,
        }
        let x: Data = serde_json::from_str(src).expect("failed to deserialize");
        assert_eq!(x.guid.typ(), "foo");
        assert_eq!(x.guid.id(), "bar");
        let ser = serde_json::to_string(&x).expect("failed to serialize");
        assert_eq!(ser, fmtjson(&src), "json does not match");
    }

    fn fmtjson(json: &str) -> String {
        serde_json::to_string(
            &serde_json::from_str::<serde_json::Value>(&json).expect("failed to parse"),
        )
        .expect("failed to serialize")
    }
}
