use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::Uuid;

pub type DateTime = chrono::DateTime<chrono::Utc>;

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
pub struct Meta {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revision: Option<Uuid>,
    #[serde(serialize_with = "ser_datetime_seconds")]
    pub created_at: DateTime,
    #[serde(serialize_with = "ser_datetime_seconds")]
    pub updated_at: DateTime,
    // publisher: Option<Guid>,
}

pub fn ser_datetime_seconds<S>(
    dt: &chrono::DateTime<chrono::Utc>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    let string = dt.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);
    string.serialize(serializer)
}

impl Default for Meta {
    fn default() -> Self {
        let now = chrono::Local::now();
        let now: DateTime = now.into();
        Self {
            revision: None,
            created_at: now.clone(),
            updated_at: now,
        }
    }
}

// pub type Root = Arc<str>;
// pub type Schema = Arc<str>;
// pub type Id = Arc<str>;

// pub struct Pubkey([u8; 32]);

// #[derive(Serialize, Deserialize)]
// pub struct TypName(String);

// pub struct Rev(Uuid);
// pub struct Id(Uuid);
// pub struct InstanceId(Uuid);
// pub struct Typ(String);

// pub struct Signature {
//     version: u16,
//     sig: [u8; 32],
// }

// struct Meta {
//     typ: Typ,
//     id: Id,
//     instance: InstanceId,
//     rev: Rev,
//     created_at: DateTime,
//     updated_at: DateTime,
//     parents: Vec<Rev>,
//     author: Vec<Id>,
// }

// addresses are instance:rev or typ:id

// revs [seq, rev]
// records [typ, id, host, rev, created_at, updated_at]
// rels [left_id, right_id, field]

// struct Ref {
//     id: Uuid,
//     typ: TypName,
//     record: Option<Record>,
// }

// struct TypedRef<T> {
//     id: Uuid,
//     record: Option<Record>,
//     typ: PhantomData<T>,
// }

// impl<T> TypedRef<T>
// where
//     T: RecordValue,
// {
//     fn to_ref(&self) -> Ref {
//         Ref {
//             id: self.id,
//             record: self.record,
//             typ: T::name(),
//         }
//     }
// }
