use std::collections::HashMap;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use uuid::Uuid as RawUuid;

pub const SEPERATOR: &str = ":";

// #[derive(Serialize, Deserialize, JsonSchema, Debug, Clone, Eq, PartialEq)]
// struct Typ(String);

#[derive(JsonSchema, Debug, Clone, Eq, PartialEq)]
pub struct Uuid(RawUuid);

impl Uuid {
    pub fn new_v4() -> Self {
        Self(RawUuid::new_v4())
    }

    pub fn new_v5(ns: &RawUuid, input: &[u8]) -> Self {
        Self(RawUuid::new_v5(ns, input))
    }

    pub fn as_bytes(&self) -> &[u8] {
        self.0.as_bytes()
    }

    pub fn to_b32(&self) -> String {
        base32::encode(base32::Alphabet::Crockford, self.as_bytes()).to_lowercase()
    }

    pub fn from_str(s: &str) -> Option<Self> {
        let uuid = s.parse::<RawUuid>().ok()?;
        Some(Self(uuid))
    }

    pub fn from_b32(data: &str) -> Option<Self> {
        let bytes = base32::decode(base32::Alphabet::Crockford, data)?;
        let bytes: [u8; 16] = bytes.try_into().ok()?;
        let uuid = RawUuid::from_bytes(bytes);
        Some(Self(uuid))
    }
}

impl Serialize for Uuid {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
        // let string = self.to_b32();
        // string.serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Uuid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn de_error<E: serde::de::Error>(e: uuid::Error) -> E {
            E::custom(format_args!("UUID parsing failed: {}", e))
        }
        struct UuidB32Visitor;
        impl<'de> serde::de::Visitor<'de> for UuidB32Visitor {
            type Value = Uuid;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid base32 encoded UUID string")
            }
            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                // if let Some(uuid) = Uuid::from_b32(v) {
                //     Ok(uuid)
                // } else {
                // }
                match value.parse::<RawUuid>() {
                    Ok(uuid) => Ok(uuid.into()),
                    Err(_) => Uuid::from_b32(value)
                        .ok_or_else(|| E::custom(format!("invalid UUID string"))),
                }
            }
            fn visit_bytes<E: serde::de::Error>(self, value: &[u8]) -> Result<Uuid, E> {
                RawUuid::from_slice(value)
                    .map_err(de_error)
                    .map(|u| Uuid(u))
            }
        }
        deserializer.deserialize_str(UuidB32Visitor)
    }
}

impl From<RawUuid> for Uuid {
    fn from(uuid: RawUuid) -> Self {
        Self(uuid)
    }
}

/// A Guid identifies a record. It contains a type string and a random UUID.
// #[derive(JsonSchema, Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
#[derive(Serialize, JsonSchema, Debug, Clone, Eq, PartialEq)]
pub struct Guid {
    id: Uuid,
    typ: String,
}

impl Guid {
    pub fn new(typ: impl ToString, id: Uuid) -> Self {
        Self {
            typ: typ.to_string(),
            id,
        }
    }

    pub fn create(typ: String) -> Self {
        Self::new(typ, uuid())
    }

    pub fn from_str(typ: String, id_payload: &str) -> Self {
        Self::new(typ, uuid_from_str(id_payload))
    }

    pub fn id(&self) -> &Uuid {
        &self.id
    }

    pub fn typ(&self) -> &str {
        &self.typ
    }

    pub fn to_guid_string(&self) -> String {
        format!("{}{}{}", self.typ, SEPERATOR, self.id.to_b32())
    }

    pub fn from_guid_str(s: &str) -> Option<Self> {
        if s.starts_with(SEPERATOR) || s.ends_with(SEPERATOR) {
            return None;
        }
        let parts: Vec<&str> = s.split_terminator(SEPERATOR).collect();
        match parts.as_slice() {
            [typ, id] if !typ.is_empty() && !id.is_empty() => Some(Self {
                typ: typ.to_string(),
                id: Uuid::from_b32(id)?,
            }),
            _ => None,
        }
    }
}

// impl Serialize for Guid {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: serde::Serializer,
//     {
//         let string = self.to_guid_string();
//         serializer.serialize_str(&string)
//     }
// }
impl<'de> Deserialize<'de> for Guid {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        fn custom_error<E: serde::de::Error>() -> E {
            E::custom(format!("Invalid GUID string"))
        }
        struct GuidVisitor;
        impl<'de> serde::de::Visitor<'de> for GuidVisitor {
            type Value = Guid;
            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a valid GUID string in the form \"typ:uuid\"")
            }
            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                Guid::from_guid_str(v).ok_or_else(custom_error)
            }

            fn visit_map<M>(self, mut access: M) -> Result<Self::Value, M::Error>
            where
                M: serde::de::MapAccess<'de>,
            {
                let mut map: HashMap<String, String> =
                    HashMap::with_capacity(access.size_hint().unwrap_or(0));

                // While there are entries remaining in the input, add them
                // into our map.
                while let Some((key, value)) = access.next_entry()? {
                    map.insert(key, value);
                }

                let uuid = map.get("id").ok_or_else(custom_error)?;
                let typ = map.get("typ").ok_or_else(custom_error)?;
                let uuid = Uuid::from_str(uuid).ok_or_else(custom_error)?;
                let guid = Guid::new(typ, uuid);

                Ok(guid)
            }
        }
        deserializer.deserialize_any(GuidVisitor)
    }
}

impl From<(String, Uuid)> for Guid {
    fn from((typ, id): (String, Uuid)) -> Self {
        Self::new(typ, id)
    }
}

pub fn uuid() -> Uuid {
    Uuid::new_v4()
}

pub fn uuid_from_str(input: &str) -> Uuid {
    const UUID_PREFIX: &str = "arso.xyz/uuid/";
    Uuid::new_v5(
        &RawUuid::NAMESPACE_URL,
        format!("{}{}", UUID_PREFIX, input).as_bytes(),
    )
}
