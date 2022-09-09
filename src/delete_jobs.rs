use std::fs::File;
use std::io::Write;
use actix_web::{delete, Responder, HttpResponse, web};
use crate::stru::*;
use crate::JOB_LIST;

#[delete("/jobs/{jobid}")]
pub async fn delete_jobs(jobid : web::Path<usize>) -> impl Responder {
    let job_id = jobid.into_inner();
    let mut lock = JOB_LIST.lock().unwrap();
    // 没有该JOB
    if job_id >= lock.len() {
        drop(lock);
        return HttpResponse::NotFound().json(
            Error {
                code : 3,
                reason : String::from("ERR_NOT_FOUND"),
                message : String::from(format!("Job {} not found.", job_id))
            }
        )
    }
    //该JOB不处在Queueing
    if lock[job_id].state != State::Queueing {
        drop(lock);
        return HttpResponse::BadRequest().json(
            Error {
                code : 2,
                reason : String::from("ERR_INVALID_STATE"),
                message : String::from(format!("Job {} not queuing.", job_id))
            }
        )
    }

    lock[job_id].state = State::Canceled;
    lock[job_id].result = Result::Skipped;
    let job_list = lock.clone();
    drop(lock);
    
    // 更新JOB_LIST文件
    let temp = serde_json::to_string_pretty(&job_list);
    let mut f = File::create("src/data/job.json").unwrap();
    match f.write_all(temp.unwrap().as_bytes()) {
        Err(_) => unimplemented!(),
        _ => {}
    }
    
    HttpResponse::Ok().json({})
}