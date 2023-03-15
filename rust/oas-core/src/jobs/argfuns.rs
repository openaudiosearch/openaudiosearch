use crate::CouchDB;
use std::collections::HashMap;
use std::fmt;
use std::future::Future;
use std::pin::Pin;

mod defaults;

#[derive(Clone, Debug)]
pub struct ArgFunContext {
    pub db: CouchDB,
}

pub type ArgFun = Box<
    dyn Fn(
            ArgFunContext,
            serde_json::Value,
        )
            -> Pin<Box<dyn Future<Output = anyhow::Result<serde_json::Value>> + Send + 'static>>
        + Send
        + Sync
        + 'static,
>;

#[derive(Default)]
pub struct ArgFunctions {
    functions: HashMap<String, ArgFun>,
}

impl fmt::Debug for ArgFunctions {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "ArgFunctions({:?})", self.functions.keys())
    }
}

impl ArgFunctions {
    pub fn with_defaults() -> Self {
        let mut this = Self::default();
        this.insert_static(
            "post_id_from_media".to_string(),
            &defaults::post_id_from_media,
        );
        this
    }
    pub fn insert(&mut self, name: String, f: ArgFun) {
        self.functions.insert(name, f);
    }

    pub fn insert_static<F, R>(&mut self, name: String, f: &'static F)
    where
        F: 'static + Send + Sync + Fn(ArgFunContext, serde_json::Value) -> R,
        R: 'static + Send + Future<Output = anyhow::Result<serde_json::Value>>,
    {
        let boxed_fun: ArgFun = Box::new(move |ctx, args| {
            let r = (f)(ctx, args);
            Box::pin(async move { r.await })
        });
        self.insert(name, boxed_fun);
    }

    pub async fn apply(
        &self,
        ctx: ArgFunContext,
        name: &str,
        input: serde_json::Value,
    ) -> anyhow::Result<serde_json::Value> {
        let fun = self
            .functions
            .get(name)
            .ok_or_else(|| anyhow::anyhow!("Argument function not found: `{}`", name))?;
        let res = (fun)(ctx, input).await;
        res
    }
}
