use std::fs::File;
use std::io::Write;

use actix_web::{Responder, HttpResponse, post, web};
use crate::USER_LIST;
use crate::stru::*;
//use crate::config::*;
use crate::CONTEST_LIST;

#[post("/contests")]
pub async fn post_contests(item: web::Json<Contest>,  config: web::Data<NewConfig>) -> impl Responder {
    // 判断用户是否存在
    let lock = USER_LIST.lock().unwrap();
    let user_list = lock.clone();
    drop(lock);
    //let mut is_ids_qualified : bool = true;
    for id in &item.user_ids {
        let mut is_qualified = false;
        for user in &user_list {
            if &user.id.unwrap() == id {
                is_qualified = true;
                break;
            }
        }
        if !is_qualified {
            return HttpResponse::NotFound().json(
                Error{
                    code : 3,
                    reason : String::from("ERR_NOT_FOUND"),
                    message : String::from(format!("User {} not found.", id))
                }
            )
        }
    }

    // 判断题目是否存在
    for id in &item.problem_ids {
        let mut is_qualified = false;
        for problem in &config.config.problems {
            if &problem.id == id {
                is_qualified = true;
                break;
            }
        }
        if !is_qualified {
            return HttpResponse::NotFound().json(
                Error{
                    code : 3,
                    reason : String::from("ERR_NOT_FOUND"),
                    message : String::from(format!("Problem {} not found.", id))
                }
            )
        }
    }
    
    match item.id {
        None => {
            // id不存在 新建contest
            let mut contest_cloned = item.clone();
            let mut lock = CONTEST_LIST.lock().unwrap();
            contest_cloned.id = Some(lock.len() as u64 +1);
            let contest_to_return = contest_cloned.clone();
            lock.push(contest_cloned);
            let contest_list = lock.clone();
            drop(lock);
            // 将contest_list写入文件
            let temp = serde_json::to_string_pretty(&contest_list);
            let mut f = File::create("src/data/contest.json").unwrap();
            match f.write_all(temp.unwrap().as_bytes()) {
                Err(_) => unimplemented!(),
                _ => {}
            }
            return HttpResponse::Ok().json(contest_to_return);
        }
        Some(id) => {
            // id存在 更新contest
            let lock = CONTEST_LIST.lock().unwrap();
            let contest_list = lock.clone();
            drop(lock);

            // id对应的比赛不存在
            if id > contest_list.len() as u64 {
                return HttpResponse::NotFound().json(
                    Error {
                        code : 3,
                        reason : String::from("ERR_NOT_FOUND"),
                        message : String::from(format!("Contest {} not found.", id))
                    }
                );
            }
            let mut lock = CONTEST_LIST.lock().unwrap();
            lock[id as usize-1] = item.clone();
            let contest_list = lock.clone();
            drop(lock);
            // 将contest_list写入文件
            let temp = serde_json::to_string_pretty(&contest_list);
            let mut f = File::create("src/data/contest.json").unwrap();
            match f.write_all(temp.unwrap().as_bytes()) {
                Err(_) => unimplemented!(),
                _ => {}
            }
            return HttpResponse::Ok().json(item); 
        }
    }
}