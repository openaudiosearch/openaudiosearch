use rocket::serde::json::Json;
use rocket::{get, patch, post};
use rocket_okapi::openapi;

use crate::server::auth::AdminUser;
use crate::server::error::Result;

use crate::jobs::{JobCreateRequest, JobId, JobInfo, JobRequest, JobResults};

/// Get a job by ID.
#[openapi(skip)]
#[get("/job/<id>")]
pub async fn get_job(state: &rocket::State<crate::State>, id: u64) -> Result<JobInfo> {
    let info = state.jobs.get_job(id).await?;
    Ok(Json(info))
}

/// Create a new job.
#[openapi(skip)]
#[post("/job", data = "<value>")]
pub async fn post_job(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    value: Json<JobCreateRequest>,
) -> Result<JobId> {
    let res = state.jobs.create_job(value.into_inner()).await?;
    Ok(Json(res))
}

/// Pull a job to work it.
// #[openapi(tag = "Job")]
#[openapi(skip)]
#[post("/work/<typ>")]
pub async fn work_job(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    typ: String,
) -> Result<JobRequest> {
    let job = state.jobs.next_job(&typ).await?;
    Ok(Json(job))
}

/// Update a job.
#[openapi(skip)]
#[patch("/job/<id>", data = "<value>")]
pub async fn patch_job(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    id: u64,
    value: Json<JobResults>,
) -> Result<()> {
    state.jobs.apply_results(id, value.into_inner()).await?;
    Ok(Json(()))
}
