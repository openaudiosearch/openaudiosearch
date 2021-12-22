use super::*;
use oas_common::types::{Media, Post};

pub async fn post_id_from_media(
    ctx: ArgFunContext,
    args: serde_json::Value,
) -> anyhow::Result<serde_json::Value> {
    let db = &ctx.db;
    let id = args
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Argument must be a string."))?;

    let record = db.table::<Media>().get(&id).await?;
    for post_ref in record.value.posts.iter() {
        let post = db.table::<Post>().get(post_ref.id()).await;
        if let Ok(post) = post {
            return Ok(serde_json::Value::String(post.id().to_string()));
        }
    }
    Err(anyhow::anyhow!("Media has no post"))
}
