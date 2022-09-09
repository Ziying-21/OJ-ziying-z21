use actix_web::{Responder, HttpResponse, web::{Query}, get };
use chrono::{NaiveDateTime};
use serde_derive::{Deserialize, Serialize};
use crate::{stru::*, USER_LIST};
use crate::JOB_LIST;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Info {
    user_id : Option<u64>,
    user_name : Option<String>,
    contest_id : Option<u64>,
    problem_id : Option<u64>,
    language : Option<String>,
    from : Option<String>,
    to : Option<String>,
    state : Option<State>,
    result : Option<Result>
}


#[get("/jobs")]
pub async fn get_jobs(info : Query<Info>) -> impl Responder{
    //println!("{:?}", info);

    // 判断时间是否符合格式
    let fmt = "%Y-%m-%dT%H:%M:%S%.3fZ";
    let is_from_present : bool;
    let mut from_time : NaiveDateTime = NaiveDateTime::default();
    let is_to_present : bool;
    let mut to_time : NaiveDateTime = NaiveDateTime::default();

    // 判断from是否符合格式
    match &info.0.from {
        None => {
            is_from_present = false;
        }
        _ => {
            //is_from_present = true;
            //let fmt = "%Y-%m-%dT%H:%M:%S%.3fZ";
            let from_time_op = NaiveDateTime::parse_from_str(&info.0.from.as_ref().unwrap()[..], fmt);
            match from_time_op {
                Err(_) => {
                    return HttpResponse::BadRequest().json(Error{
                        code : 1,
                        reason : String::from("ERR_INVALID_ARGUMENT"),
                        message : String::from(format!("Invalid argument from={}", &info.0.from.as_ref().unwrap()[..]))
                    });
                }
                _ => {
                    from_time = from_time_op.unwrap();
                }
            }
            is_from_present = true;
        }
    }

    // 判断to是否符合格式
    match &info.0.to {
        None => {
            is_to_present = false;
        }
        _ => {
            //let fmt = "%Y-%m-%dT%H:%M:%S%.3fZ";
            let to_time_op = NaiveDateTime::parse_from_str(&info.0.to.as_ref().unwrap()[..], fmt);
            match to_time_op {
                Err(_) => {
                    return HttpResponse::BadRequest().json(Error{
                        code : 1,
                        reason : String::from("ERR_INVALID_ARGUMENT"),
                        message : String::from(format!("Invalid argument to={}", &info.0.to.as_ref().unwrap()[..]))
                    });
                }
                _ => {
                    to_time = to_time_op.unwrap();
                }
            }
            is_to_present = true;
        }
    }

    // 按条件筛选任务
    let mut qualified_job_list : Vec<Task> = Vec::new();
    let list_length : usize;
    let lock = JOB_LIST.lock().unwrap();
    list_length = lock.len();
    drop(lock);
    for i in 0..list_length {
        let temp_task;
        let lock = JOB_LIST.lock().unwrap();
        temp_task = lock[i].clone();
        drop(lock);
        //let is_qualified = true;
        // 根据比赛ID进行筛选
        match info.contest_id {
            None => {}
            Some(id) => {
                if temp_task.submission.contest_id != id {
                    continue;
                }
            }
            
        }
        // 根据用户ID进行筛选
        match info.user_id {
            None => {}
            Some(user_id) =>{
                if temp_task.submission.user_id != user_id {
                    continue;
                }
            }
        }
        // 根据用户名进行筛选
        match &info.user_name {
            None => {}
            Some(user_name) => {
                let lock = USER_LIST.lock().unwrap();
                if &lock[temp_task.submission.user_id as usize].name != user_name {
                    continue;
                }
            }
        }
        // 根据问题ID进行筛选
        match info.0.problem_id {
            None => {}
            Some(id) => {
                if temp_task.submission.problem_id != id {
                    continue;
                }
            }
        }
        // 根据语言进行筛选
        match info.0.language {
            None => {}
            Some(ref lang) => {
                if &temp_task.submission.language != lang {
                    continue;
                }
            }
        }
        // 根据开始时间进行筛选
        if is_from_present {
            let task_time = NaiveDateTime::parse_from_str(&temp_task.created_time[..], fmt);
            if task_time.unwrap() < from_time {
                continue;
            }
        }    
        // 根据截至时间进行筛选
        if is_to_present {
            let task_time = NaiveDateTime::parse_from_str(&temp_task.created_time[..], fmt);
            if task_time.unwrap() > to_time {
                continue;
            }
        } 
        // 根据状态进行筛选
        match &info.0.state {
            None => {}
            Some(ref s) => {
                if &temp_task.state != s {
                    continue;
                }
            }
        }
        // 根据结果进行筛选
        match &info.0.result {
            None => {}
            Some(ref r) => {
                if &temp_task.result != r {
                    continue;
                }
            }
        }

        // 符合所有条件
        qualified_job_list.push(temp_task);
    }
    HttpResponse::Ok().json(qualified_job_list)
}