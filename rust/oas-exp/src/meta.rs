use crate::RecordValue;
use crate::Uuid;
use serde::{Deserialize, Serialize};

pub type DateTime = chrono::DateTime<chrono::Utc>;

pub struct Meta {
    created_at: DateTime,
    updated_at: DateTime,
    publisher: Option<Guid>,
    revision: Option<String>,
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
