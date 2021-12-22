use schemars::JsonSchema;

use serde::{Deserialize, Serialize};
use serde_json::json;
use std::any::Any;

use oas_exp::*;

fn main() {
    if let Err(err) = run_all() {
        panic!("{}", err)
    }
}
fn run_all() -> anyhow::Result<()> {
    run_inventory()?;
    run()
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
struct Media {
    url: String,
}

impl AsAny for Media {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn AsAny> {
        Box::new(self.clone())
    }
}

impl RecordValue for Media {
    const NAME: &'static str = "media";
}

#[derive(Serialize, Deserialize, Debug, Clone, JsonSchema)]
struct Post {
    headline: String,
}

impl AsAny for Post {
    fn as_any(&self) -> &dyn Any {
        self
    }
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
    fn clone_box(&self) -> Box<dyn AsAny> {
        Box::new(self.clone())
    }
}
impl RecordValue for Post {
    const NAME: &'static str = "post";
}

pub fn run_inventory() -> anyhow::Result<()> {
    let mut inventory = Inventory::new();
    inventory.insert(Schema::from_value_typ::<Media>());
    inventory.insert(Schema::from_value_typ::<Post>());

    let json1 = json!({
        "id": uuid_from_str("abc"),
        "typ": "media",
        "value": {
            "url": "hyper://foo"
        }
    });
    let json2 = json!({
        "id": uuid_from_str("def"),
        "typ": "post",
        "value": {
            "headline": "Foobar foo"
        }
    });
    let json3 = json!({
        "id": uuid_from_str("def"),
        "typ": "foo",
        "value": {
            "headline": "Foobar foo"
        }
    });
    let json4 = json!({
        "id": uuid_from_str("def"),
        "typ": "post",
        "value": {
            "boo": "bazz"
        }
    });

    let jsons = vec![json1.clone(), json2.clone(), json3, json4];

    let mut records: Vec<_> = jsons
        .clone()
        .into_iter()
        .map(|j| Record::from_json(j).unwrap())
        .collect();
    let upcast_results: Vec<_> = records.iter_mut().map(|r| inventory.upcast(r)).collect();
    eprintln!(
        "list upcasted: jsons {:#?}, records {:#?}, upcast res {:#?}",
        jsons, records, upcast_results
    );

    //
    let mut record1 = Record::from_json(json1)?;
    let mut record2 = Record::from_json(json2)?;
    eprintln!("pre1 {:?}", record1);
    eprintln!("pre2 {:?}", record2);
    let r1 = inventory.upcast(&mut record1);
    let r2 = inventory.upcast(&mut record2);
    eprintln!("r1 {:?}", r1);
    eprintln!("r2 {:?}", r2);
    eprintln!("rec1 upcasted {:?}", record1);
    eprintln!("rec2 upcasted {:?}", record2);

    eprintln!("res r1 upcast media {:?}", record1.upcast::<Media>());
    eprintln!("res r1 upcast post {:?}", record1.upcast::<Post>());
    eprintln!("res r2 upcast media {:?}", record2.upcast::<Media>());
    eprintln!("res r2 upcast post {:?}", record2.upcast::<Post>());

    Ok(())
}

pub fn run() -> anyhow::Result<()> {
    let json = json!({
        "id": uuid_from_str("abc"),
        "typ": "media",
        "value": {
            "url": "hyper://foo"
        }
    });
    let mut record = Record::from_json(json)?;
    record.upcast::<Media>()?;

    let media = record.value::<Media>()?;
    eprintln!("typed media {:?}", media);
    let post_res = record.value::<Post>();
    eprintln!("typed post res {:?}", post_res);
    let media: &Media = record.value()?;
    eprintln!("typed {:?}", media);
    let media = media.clone();
    eprintln!("typed {:?}", media);

    let ser = record.into_json_cloned()?;
    eprintln!("json {:?}", ser);

    let mut media = record.value_mut::<Media>()?;
    media.url = "hyper:://bar".to_string();
    let ser = record.value_as_json()?;
    eprintln!("json {:?}", ser);

    let blank = record.blank();
    eprintln!("blank {:?}", blank);
    // eprintln!("blank typed {:?}", blank.typed()?);
    let unblank = {
        let mut blank = blank;
        blank.upcast::<Media>()?;
        blank
    };
    eprintln!("unblank {:?}", unblank);
    // TODO!!
    // eprintln!("unblank typed {:?}", unblank.value::<Blank>()?);

    eprintln!("\n\nround 2\n\n");
    let json = unblank.into_json_cloned()?;
    eprintln!("json {}", json);
    let mut record = Record::from_json(json)?;
    eprintln!("blank {:?}", record);
    record.upcast::<Media>()?;
    let media = record;
    eprintln!("unblank {:?}", media);
    let value = media.value::<Media>()?.clone();
    eprintln!("media {:?}", value);

    let record = Record::new(uuid_from_str("newone"), value);
    eprintln!("record {:?}", record);
    let json = record.into_json()?;
    eprintln!("json {}", json);

    eprintln!("\n\nround3\n\n");
    let media: Record = Record::from_json(json.clone())?;
    eprintln!("media {:?}", media);
    let blank: Record = Record::from_json(json)?;
    eprintln!("blank {:?}", blank);
    let media = blank.into_upcast::<Media>()?;
    eprintln!("upcast media {:?}", media);
    // let media = blank.upcast_lucky::<Post>();
    // eprintln!("upcast media lucky {:?}", media);

    let reblank = media.blank();
    eprintln!("reblank {:?}", reblank);
    let mut media = reblank.into_upcast::<Media>()?;
    eprintln!("upcast media {:?}", media.value::<Media>());
    media.meta_mut().revision = Some(uuid());
    let json = serde_json::to_string(&media)?;
    eprintln!("media as json {}", json);

    // eprintln!("\n\nround4\n\n");
    // let media = media;
    // let post = Record::new(
    //     "post1".into(),
    //     Post {
    //         headline: "hi there".into(),
    //     },
    // );

    // let mut store = RecordStore::new();
    // store.insert::<Media>(media);
    // store.insert::<Post>(post);

    // let blank = Record::from_json(json!({
    //     "typ": "post",
    //     "id": "post2",
    //     "value": {"headline": "ayay!"}
    // }))?;

    // store.insert_blank(blank);

    // eprintln!("get media {:?}", store.get::<Media>("newone"));
    // eprintln!("get post {:?}", store.get::<Media>("post1"));
    // eprintln!("get post {:?}", store.get::<Post>("post1"));
    // eprintln!("get post {:?}", store.get::<Post>("post2"));

    // eprintln!("bref post {:?}", bref.value::<Media>());

    // let post: Record<Post> = post.upcast()?.typed(;
    // eprintln!("unblank {:?}", post);
    // eprintln!("unblank val {:?}", post.typed());

    Ok(())
}
