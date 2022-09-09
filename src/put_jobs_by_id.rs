use std::fs::File;
use std::io::Write;
use actix_web::{put, Responder, HttpResponse, web};
use crate::JOB_LIST;
use crate::stru::*;
use crate::config::*;

#[put("/jobs/{jobid}")]
pub async fn put_jobs_by_id(jobid : web::Path<usize>, config: web::Data<NewConfig>) -> impl Responder{
    let id : usize = jobid.into_inner();
    let lock = JOB_LIST.lock().unwrap();
    let list_len = lock.len();
    drop(lock);
    for i in 0..list_len {
        // 遍历所有job
        let lock = JOB_LIST.lock().unwrap();
        let task_checked = lock[i].clone();
        drop(lock);
        if task_checked.id == id {
            // 该job尚未完成
            if task_checked.state != State::Finished {
                return HttpResponse::BadRequest().json(
                    Error {
                        code : 2,
                        reason : String::from("ERR_INVALID_STATE"),
                        message : String::from(format!("Job {} not finished.", id))
                    }
                )
            }
            
            // 该job已完成 可以重测
            let submission = task_checked.submission;
            
            // 检查编程语言是否在配置中
            let mut lang : Language = Language::new();
            for language in &config.config.languages {
                if submission.language == language.name {
                    //is_language_in = true;
                    lang = language.clone();
                    break;
                }
            }
            /*
            if !is_language_in {
                return HttpResponse::NotFound().json(Error{
                    code : 2,
                    reason : String::from("ERR_NOT_FOUND"),
                    message : format!("Language {} not found.", submission.language.to_string())
                })
            }
            */
            // 检查题目ID是否在配置中
            //let mut is_problem_id_in = false;
            let mut pro : Problem = Problem::new();
            for problem in &config.config.problems {
                if submission.problem_id == problem.id {
                    //is_problem_id_in = true;
                    pro = problem.clone();
                    break;
                }
            }
            /*
            if !is_problem_id_in {
                return HttpResponse::NotFound().json(Error{
                    code : 2,
                    reason : String::from("ERR_NOT_FOUND"),
                    message : format!("Problem {} not found.", submission.problem_id)
                })
            }
            */
            let mut lock = JOB_LIST.lock().unwrap();
            //lock[i].state = State::Running;
            //lock[i].result = Result::Running;
            // 重置任务状态
            lock[i].state = State::Queueing;
            lock[i].result = Result::Waiting;
            lock[i].score = 0.0;
            // 重置各数据点状态
            for j in 0..lock[i].cases.len() {
                lock[i].cases[j] = Case::new();
                lock[i].cases[j].id = j as u64;
            }
            let task_clone = lock[i].clone();
            drop(lock);
            
            /*
            // 新建线程进行评测
            thread::spawn(move || {
                evaluate(&pro, &submission, &lang, i);
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
                submission : submission,
                index : i
            };
            let sender = config.sender.clone();
            match sender.unwrap().send(evaluate_para) {
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
            return HttpResponse::Ok().json(task_clone);
        }
    }
    // 未找到
    HttpResponse::NotFound().json(
        Error{
            code : 3,
            reason : String::from("ERR_NOT_FOUND"),
            message : String::from(format!("Job {} not found", id))
        }
    )
}