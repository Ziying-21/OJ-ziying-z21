use std::fs::File;
use std::io::Write;

use actix_web::{Responder, post, HttpResponse, web};
use crate::USER_LIST;
use crate::stru::*;


#[post("/users")]
pub async fn post_users(item: web::Json<User>) -> impl Responder {
    match item.id {
        None => {
            // id字段不存在,新建用户
            let mut lock = USER_LIST.lock().unwrap();
            for user in lock.iter() {
                if user.name == item.name {
                    // 找到重名
                    drop(lock);
                    return HttpResponse::BadRequest().json(
                        Error{
                            code : 1,
                            reason : String::from("ERR_INVALID_ARGUMENT"),
                            message : String::from(format!("User name '{}' already exists", item.name))
                        }
                    )
                }
            }
            // 无重名
            let new_user = User {
                id : Some(lock.len() as u64),
                name : item.name.to_string()
            };
            let cloned = new_user.clone();
            lock.push(new_user);
            let user_list = lock.clone();
            drop(lock);
            let temp = serde_json::to_string_pretty(&user_list);
            let mut f = File::create("src/data/user.json").unwrap();
            match f.write_all(temp.unwrap().as_bytes()) {
                Err(_) => unimplemented!(),
                _ => {}
            }
            return HttpResponse::Ok().json(cloned);
        }
        _ => {
            // id 字段存在,用户改名
            let mut lock = USER_LIST.lock().unwrap();
            if item.id.unwrap() >= lock.len() as u64 {
                // 表中没有这个id
                drop(lock);
                return HttpResponse::NotFound().json(
                    Error{
                        code : 3,
                        reason : String::from("ERR_NOT_FOUND"),
                        message : String::from(format!("User {} not found.", item.id.unwrap()))
                    }
                );
            }
            for user in lock.iter() {
                if (user.name == item.name)&&(user.id != item.id) {
                    // 找到其他id且重名
                    drop(lock);
                    return HttpResponse::BadRequest().json(
                        Error{
                            code : 1,
                            reason : String::from("ERR_INVALID_ARGUMENT"),
                            message : String::from(format!("User name '{}' already exists", item.name))
                        }
                    )
                }
            }
            // 找到id且无重名
            lock[item.id.unwrap() as usize].name = item.name.clone(); 
            let cloned = lock[item.id.unwrap() as usize].clone();
            let user_list = lock.clone();
            drop(lock);
            let temp = serde_json::to_string_pretty(&user_list);
            let mut f = File::create("src/data/user.json").unwrap();
            match f.write_all(temp.unwrap().as_bytes()) {
                Err(_) => unimplemented!(),
                _ => {}
            }
            return HttpResponse::Ok().json(cloned);
        }
    }
    //HttpResponse::Ok().json(item)
}