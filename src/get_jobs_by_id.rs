use actix_web::{Responder, HttpResponse, get, web};
use crate::stru::*;
use crate::JOB_LIST;

#[get("/jobs/{jobid}")]
async fn get_jobs_by_id(jobid : web::Path<usize>) -> impl Responder {
    let lock = JOB_LIST.lock().unwrap();
    let list_len = lock.len();
    let id = jobid.into_inner();
    drop(lock);
    for i in 0..list_len {
        let lock = JOB_LIST.lock().unwrap();
        let task = lock[i].clone();
        drop(lock);
        if task.id == id {
            // 找到该id
            return HttpResponse::Ok().json(
                task
            )
        }
    }
    // 未找到
    HttpResponse::NotFound().json(Error{
        code : 3,
        reason : String::from("reason=ERR_NOT_FOUND"),
        message : String::from(format!("Job {} not found.", id))
    })
}