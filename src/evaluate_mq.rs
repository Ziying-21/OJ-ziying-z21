use crossbeam;
use crate::stru::*;
use crate::JOB_LIST;
use crate::evaluate::*;
pub fn evaluate_mq (receiver : crossbeam::channel::Receiver<EvaluatePara>) {
    loop {
        let r = receiver.try_recv();
        match r {
            Err(_) => {}
            Ok(para) => {
                // 判断该任务是否已经取消,取消则不再评测,否则继续评测
                let lock = JOB_LIST.lock().unwrap();
                let state = lock[para.index].state.clone();
                drop(lock);
                if state == State::Queueing {
                    evaluate(&para.problem, &para.submission, &para.language, para.index);
                }
            }
        }
    } 
}