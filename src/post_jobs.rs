use std::fs::File;
use std::io::Write;
//use std::os::unix::thread;

use actix_web::post;
use actix_web::{web, Responder, HttpResponse};
use chrono::{DateTime, Utc};
//use crossterm::event::poll;
use crate::{stru::*, USER_LIST, CONTEST_LIST};
use crate::config::*;
use crate::{ JOB_LIST };


#[post("/jobs")]
pub async fn post_jobs(item: web::Json<Submission>, config: web::Data<NewConfig>) -> impl Responder {
    // 检查编程语言是否在配置中
    let mut is_language_in = false;
    let mut lang : Language = Language::new();
    for language in &config.config.languages {
        if item.language == language.name {
            is_language_in = true;
            lang = language.clone();
            break;
        }
    }
    if !is_language_in {
        return HttpResponse::NotFound().json(Error{
            code : 3,
            reason : String::from("ERR_NOT_FOUND"),
            message : format!("Language {} not found.", item.language.to_string())
        })
    }

    // 检查题目ID是否在配置中
    let mut is_problem_id_in = false;
    let mut pro : Problem = Problem::new();
    for problem in &config.config.problems {
        if item.problem_id == problem.id {
            is_problem_id_in = true;
            pro = problem.clone();
            break;
        }
    }
    if !is_problem_id_in {
        return HttpResponse::NotFound().json(Error{
            code : 3,
            reason : String::from("ERR_NOT_FOUND"),
            message : format!("Problem {} not found.", item.problem_id)
        })
    }
    // 检查用户ID是否存在
    let lock = USER_LIST.lock().unwrap();
    if item.user_id >= lock.len() as u64 {
        // 用户ID不存在
        drop(lock);
        return HttpResponse::NotFound().json(Error{
            code : 3,
            reason : String::from("ERR_NOT_FOUND"),
            message : format!("User {} not found.", item.user_id)
        })
    }
    drop(lock);
    if item.contest_id != 0 {
        // 检查比赛 ID 是否存在
        let lock = CONTEST_LIST.lock().unwrap();
        if item.contest_id > lock.len() as u64 {
            drop(lock);
            return HttpResponse::NotFound().json(Error{
                code : 3,
                reason : String::from("ERR_NOT_FOUND"),
                message : format!("Contest {} not found.", item.contest_id)
            });
        }
        drop(lock);
        // 检查用户 ID 是否在此比赛中
        let lock = CONTEST_LIST.lock().unwrap();
        if !lock[item.contest_id as usize - 1].user_ids.contains(&item.user_id) {
            drop(lock);
            return HttpResponse::BadRequest().json(Error{
                code : 1,
                reason : String::from("ERR_INVALID_ARGUMENT"),
                message : String::from("User not in the contest.")
            });
        }
        drop(lock);
        // 检查题目 ID 是否在此比赛中
        let lock = CONTEST_LIST.lock().unwrap();
        if !lock[item.contest_id as usize - 1].problem_ids.contains(&item.problem_id) {
            drop(lock);
            return HttpResponse::BadRequest().json(Error{
                code : 1,
                reason : String::from("ERR_INVALID_ARGUMENT"),
                message : String::from("Problem not in the contest.")
            });
        }
        drop(lock);
        // 用户该题目的提交次数限制是否达到上限
        let mut trial_times = 0;
        let lock = JOB_LIST.lock().unwrap();
        let job_list = lock.clone();
        drop(lock);
        for job in job_list {
            if 
            (job.submission.user_id == item.user_id) && 
            (job.submission.problem_id == item.problem_id) && 
            (job.submission.contest_id == item.contest_id)
            {
                trial_times += 1;
            }
        }
        let lock = CONTEST_LIST.lock().unwrap();
        if trial_times >= lock[item.contest_id as usize -1].submission_limit &&
        lock[item.contest_id as usize -1].submission_limit != 0
        {
            drop(lock);
            return HttpResponse::BadRequest().json(
                Error{
                    code : 4,
                    reason : String::from("ERR_RATE_LIMIT"),
                    message : String::from("Submission limit exceeded")
                }
            )
        }
        drop(lock);
        // 提交评测任务时间是否在比赛进行时间范围内
        let fmt = "%Y-%m-%dT%H:%M:%S%.3fZ";
        let now : DateTime<Utc> = Utc::now();
        let now_str = now.format(fmt).to_string();
        let lock = CONTEST_LIST.lock().unwrap();
        if (now_str < lock[item.contest_id as usize -1].from) || 
        (now_str > lock[item.contest_id as usize -1].to) {
            drop(lock);
            return HttpResponse::BadRequest().json(
                Error{
                    code : 1,
                    reason : String::from("ERR_INVALID_ARGUMENT"),
                    message : String::from("Not within the competition time")
                }
            )
        }
        drop(lock);
    }
    

    // 新建任务并压入 JOB_LIST
    let mut new_task = Task::new();
    for i in 0..(pro.cases.len()+1) {
        let mut new_case = Case::new();
        new_case.id = i as u64;
        new_task.cases.push(new_case);
    }
    new_task.submission = item.0.clone();
    let fmt = "%Y-%m-%dT%H:%M:%S%.3fZ";
    let now : DateTime<Utc> = Utc::now();
    new_task.created_time = now.format(fmt).to_string();
    let mut lock = JOB_LIST.lock().unwrap();
    let index = lock.len();
    new_task.id = index;
    let task_clone = new_task.clone();
    lock.push(new_task);
    drop(lock);


    /*
    // 启动新线程进行评测
    thread::spawn(move || {
        evaluate(& pro, & item.0, & lang, index);
    });
    */

    // 将新的JOB_LIST存入json文件
    let lock = JOB_LIST.lock().unwrap();
    let job_list = lock.clone();
    drop(lock);
    let temp = serde_json::to_string_pretty(&job_list);
    let mut f = File::create("src/data/job.json").unwrap();
    match f.write_all(temp.unwrap().as_bytes()) {
        Err(_) => unimplemented!(),
        _ => {}
    }


    // 将任务压入消息队列等待评测
    let evaluate_para = EvaluatePara {
        language : lang,
        problem : pro,
        submission : item.0,
        index : index
    };
    let sender = config.sender.clone();
    
    match sender.unwrap().send(evaluate_para) {
        // 压入失败
        Err(_) => {
            return HttpResponse::InternalServerError().json(
                Error {
                    code : 6,
                    reason : String::from("ERR_INTERNAL"),
                    message : String::from("Unable to connect to the profiler")
                }
            )
        }
        Ok(_) => {}
    }
    
    HttpResponse::Ok().json(task_clone)
}