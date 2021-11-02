use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::FromForm;
use rocket::{get, post, put};
use rocket_okapi::openapi;

use crate::server::auth::AdminUser;
use crate::server::error::{AppError, Result};

use crate::jobs::{
    JobCompletedRequest, JobCreateRequest, JobFailedRequest, JobFilter, JobId, JobInfo,
    JobProgressRequest, JobRequest, JobStatus,
};

#[derive(FromForm)]
pub struct JobQuery {
    pub status: Vec<String>,
    pub queue: Vec<String>,
}

#[openapi(skip)]
#[get("/jobs?<filter..>")]
pub async fn get_all_jobs(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    filter: Option<JobQuery>,
) -> Result<Vec<JobInfo>> {
    let filter = if let Some(filter) = filter {
        let status: Vec<JobStatus> = serde_json::from_value(serde_json::to_value(filter.status)?)?;
        Some(JobFilter {
            queue: filter.queue,
            status,
        })
    } else {
        None
    };
    let jobs = state.jobs.all_jobs(filter).await?;
    Ok(Json(jobs))
}

/// Get a job by ID.
#[openapi(skip)]
#[get("/job/<id>")]
pub async fn get_job(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    id: u64,
) -> Result<JobInfo> {
    let info = state.jobs.job(id).await?;
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
) -> std::result::Result<Custom<rocket::serde::json::Json<Option<JobRequest>>>, AppError> {
    let job = state.jobs.next_job(&typ).await?;
    if let Some(job) = job {
        Ok(Custom(Status::Ok, Json(Some(job))))
    } else {
        Ok(Custom(Status::NoContent, Json(None)))
    }
}

/// Complete a job.
#[openapi(skip)]
#[put("/job/<id>/completed", data = "<value>")]
pub async fn put_job_completed(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    id: u64,
    value: Json<JobCompletedRequest>,
) -> Result<()> {
    state.jobs.set_completed(id, value.into_inner()).await?;
    Ok(Json(()))
}

/// Report progress on a job.
#[openapi(skip)]
#[put("/job/<id>/progress", data = "<value>")]
pub async fn put_job_progress(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    id: u64,
    value: Json<JobProgressRequest>,
) -> Result<()> {
    state.jobs.set_progress(id, value.into_inner()).await?;
    Ok(Json(()))
}

/// Fail a job.
#[openapi(skip)]
#[put("/job/<id>/failed", data = "<value>")]
pub async fn put_job_failed(
    _user: AdminUser,
    state: &rocket::State<crate::State>,
    id: u64,
    value: Json<JobFailedRequest>,
) -> Result<()> {
    state.jobs.set_failed(id, value.into_inner()).await?;
    Ok(Json(()))
}
