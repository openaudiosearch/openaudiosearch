use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::record::TypedValue;
// use crate::{ElasticMapping, Reference};
// use crate::mapping::Mappable;

#[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
pub struct Any(serde_json::Value);

impl TypedValue for Any {
    const NAME: &'static str = "oas.Any";
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Record, UntypedRecord};
    use serde_json::json;
    #[test]
    fn any_basics() {
        #[derive(Serialize, Deserialize, Debug, Clone, Default, JsonSchema)]
        struct Community {
            name: String,
            color: String,
        }
        impl TypedValue for Community {
            const NAME: &'static str = "Community";
        }
        let typ = "Community";
        let id = "test1";
        let value = json!({
            "name": "solarpunk",
            "color": "pink"
        });
        let record = Record::from_id_and_value(id, Any(value));
        eprintln!("record rust {:?}", record);
        eprintln!("record json {}", serde_json::to_string(&record).unwrap());
        let untyped = record.into_untyped().unwrap();
        eprintln!("untyped rust {:?}", untyped);
        eprintln!("untyped json {}", serde_json::to_string(&untyped).unwrap());
        let record = untyped.clone().into_typed::<Any>().unwrap();
        eprintln!("any rust {:?}", record);
        eprintln!("any json {}", serde_json::to_string(&record).unwrap());
        let mut untyped2 = untyped.clone();
        untyped2.meta.typ = typ.to_string();
        let record = untyped2.into_typed::<Community>().unwrap();
        eprintln!("typed rust {:?}", record);
        eprintln!("typed json {}", serde_json::to_string(&record).unwrap());
        let record = record.into_untyped().unwrap();
        eprintln!("untyped rust {:?}", record);
        eprintln!("untyped json {}", serde_json::to_string(&record).unwrap());
    }
}
