use std::io::Write;
use std::{fs::File, io::Read};
use std::sync::{Arc, Mutex};
use actix_cors::Cors;
//use crossbeam::queue::SegQueue;
//use std::time::Duration;
use lazy_static::lazy_static;
use actix_web::{ middleware::Logger, post, web::{self}, App, HttpServer, Responder};
use clap::{App as ClapApp, load_yaml};
use env_logger;
use log;
//use serde_derive::{ Serialize, Deserialize};
mod stru;
use stru::*;
mod config;
use config::*;
mod post_jobs;
use post_jobs::*;
mod get_jobs;
use get_jobs::*;
mod get_jobs_by_id;
use get_jobs_by_id::*;
mod put_jobs_by_id;
use put_jobs_by_id::*;
mod get_users;
use get_users::*;
mod post_users;
use post_users::*;
mod evaluate;
//use evaluate::evaluate;
mod get_ranklist;
use get_ranklist::*;
mod post_contests;
use post_contests::*;
mod get_contests;
use get_contests::*;
mod get_contests_by_id;
use get_contests_by_id::*;
mod delete_jobs;
use delete_jobs::*;
mod evaluate_mq;
use evaluate_mq::evaluate_mq;
mod option;
use option::options;
use std::thread;
//use std::sync::mpsc::{self};
use crossbeam;

// 保存本次启动以来所有评测任务的全局变量
lazy_static! {
    static ref JOB_LIST: Arc<Mutex<Vec<Task>>> =
    Arc::new(Mutex::new(Vec::new())); 
}
// 保存本次启动以来已新建过的评测临时文件夹数量
lazy_static! {
    static ref TASK_NUM: Arc<Mutex<u64>> =
    Arc::new(Mutex::new(0)); 
}

// 保存本次启动以来所有的用户
lazy_static! {
    static ref USER_LIST: Arc<Mutex<Vec<User>>> =
    Arc::new(Mutex::new(Vec::new())); 
}

// 保存本次启动以来所有的比赛
lazy_static! {
    static ref CONTEST_LIST: Arc<Mutex<Vec<Contest>>> =
    Arc::new(Mutex::new(Vec::new())); 
}

// DO NOT REMOVE: used in automatic testing
#[post("/internal/exit")]
#[allow(unreachable_code)]
async fn exit() -> impl Responder {
    log::info!("Shutdown as requested");
    std::process::exit(0);
    format!("Exited")
}



#[actix_web::main]
async fn main() -> std::io::Result<()> {

    // 处理配置文件
    let yml = load_yaml!("yaml.yml");
    let matches = ClapApp::from_yaml(yml).get_matches();
    let path = matches.value_of("config");
    match path {
        None => {
            panic!("No config file!");
        }
        _ => {}
    }
    let mut f = File::open(path.unwrap())?;
    let mut config : Config;
    let mut buf = String::new();
    f.read_to_string(&mut buf).unwrap();
    config = serde_json::from_str(&buf)?;
    // 添加默认信息
    match config.server.bind_address {
        None => config.server.bind_address = Some(String::from("127.0.0.1")),
        _ => {}
    }
    match config.server.bind_port {
        None => config.server.bind_port = Some(12345),
        _ => {}
    }

    let mut new_config : NewConfig = NewConfig {
        config : config.clone(),
        sender : None
    };
    
    
    // 判断是否清除持久化存储
    if matches.is_present("flush-data") {
        let s = String::from("[]");
        let mut f_1 = File::create("src/data/user.json").unwrap();
        let mut f_2 = File::create("src/data/contest.json").unwrap();
        let mut f_3 = File::create("src/data/job.json").unwrap();
        f_1.write_all(s.as_bytes())?;
        f_2.write_all(s.as_bytes())?;
        f_3.write_all(s.as_bytes())?;
    }

    // 从持久化存储中读取数据
    let mut f_1 = File::open("src/data/user.json").unwrap();
    let mut f_2 = File::open("src/data/contest.json").unwrap();
    let mut f_3 = File::open("src/data/job.json").unwrap();
    let mut buf_1 = String::new();
    let mut buf_2 = String::new();
    let mut buf_3 = String::new();
    f_1.read_to_string(&mut buf_1).unwrap();
    f_2.read_to_string(&mut buf_2).unwrap();
    f_3.read_to_string(&mut buf_3).unwrap();
    let mut lock = USER_LIST.lock().unwrap();
    let temp : Vec<User> = serde_json::from_str(&buf_1)?;
    for user in temp {
        lock.push(user);
    }
    drop(lock);
    let mut lock = CONTEST_LIST.lock().unwrap();
    let temp : Vec<Contest> = serde_json::from_str(&buf_2)?;
    for contest in temp {
        lock.push(contest);
    }
    drop(lock);
    let mut lock = JOB_LIST.lock().unwrap();
    let temp : Vec<Task> = serde_json::from_str(&buf_3)?;
    for task in temp {
        lock.push(task);
    }
    drop(lock);


    //game_state = serde_json::from_str(&buf)?;

    // 若USER_LIST为空则新建root用户
    let mut lock = USER_LIST.lock().unwrap();
    if lock.len() == 0 {
        lock.push(
            User {
                id : Some(0),
                name : String::from("root")
            }
        );
    }
    drop(lock);

    /* 每次更新三个全局变量都会顺便更新json文件,此功能已不再发挥作用
    // 新建进程定时更新data/json文件
    thread::spawn(||{
        loop {
            let lock = JOB_LIST.lock().unwrap();
            let job_list = lock.clone();
            drop(lock);
            let temp = serde_json::to_string_pretty(&job_list);
            let mut f = File::create("src/data/job.json").unwrap();
            match f.write_all(temp.unwrap().as_bytes()) {
                Err(_) => unimplemented!(),
                _ => {}
            }
            thread::sleep(Duration::from_secs(30));
            let lock = CONTEST_LIST.lock().unwrap();
            let contest_list = lock.clone();
            drop(lock);
            let temp = serde_json::to_string_pretty(&contest_list);
            let mut f = File::create("src/data/contest.json").unwrap();
            match f.write_all(temp.unwrap().as_bytes()) {
                Err(_) => unimplemented!(),
                _ => {}
            }
            thread::sleep(Duration::from_secs(30));
            let lock = USER_LIST.lock().unwrap();
            let user_list = lock.clone();
            drop(lock);
            let temp = serde_json::to_string_pretty(&user_list);
            let mut f = File::create("src/data/user.json").unwrap();
            match f.write_all(temp.unwrap().as_bytes()) {
                Err(_) => unimplemented!(),
                _ => {}
            }
            thread::sleep(Duration::from_secs(30));
        }
    });
    */

    // 三个线程同时进行评测
    // 无限容量
    // 采用crossbeam实现mpmc
    let (sender, receiver) = crossbeam::channel::unbounded::<EvaluatePara>();
    new_config.sender = Some(sender.clone());
    let receiver_1 = receiver.clone();
    let receiver_2 = receiver.clone();
    thread::spawn(move ||{
        evaluate_mq(receiver);
    });
    thread::spawn(move ||{
        evaluate_mq(receiver_1);
    });
    thread::spawn(move ||{
        evaluate_mq(receiver_2);
    });


    
    // 服务端
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(new_config.clone()))
            .wrap(Logger::default())
            //.route("/hello", web::get().to(|| async { "Hello World!" }))
            //.service(greet)
            .wrap(
                Cors::permissive()
                //.finish()
            )
            .service(
                post_jobs
            )
            .service(
                get_jobs
            )
            .service(
                get_jobs_by_id
            )
            .service(
                put_jobs_by_id
            )
            .service(
                get_users
            )
            .service(
                post_users
            )
            .service(
                get_ranklist
            )
            .service(
                post_contests
            )
            .service(
                get_contests
            )
            .service(
                get_contests_by_id
            )
            // DO NOT REMOVE: used in automatic testing
            .service(
                delete_jobs
            )
            .service(
                options
            )
            .service(exit)
    })
    .bind((config.server.bind_address.unwrap(), config.server.bind_port.unwrap()))?
    .run()
    .await
}
