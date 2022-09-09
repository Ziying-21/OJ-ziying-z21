use actix_web::{Responder, HttpResponse, get, web};
use crate::stru::*;
use crate::CONTEST_LIST;


#[get("/contests/{contestid}")]
pub async fn get_contests_by_id(contestid : web::Path<u64>) -> impl Responder {
    let contest_id = contestid.into_inner();
    let lock = CONTEST_LIST.lock().unwrap();
    let contest_list = lock.clone();
    drop(lock);
    if contest_id > contest_list.len() as u64 {
        return HttpResponse::NotFound().json(
            Error{
                code : 3,
                reason : String::from("ERR_NOT_FOUND"),
                message : String::from(format!("Contest {} not found.", contest_id))
            }
        );
    }
    HttpResponse::Ok().json(&contest_list[contest_id as usize -1])
}