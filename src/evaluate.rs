use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::io::Write;
use std::process::Command;
use chrono::DateTime;
use wait_timeout::ChildExt;
use std::time::Duration;
use chrono::Utc;
use crate::JOB_LIST;
use crate::TASK_NUM;
use crate::stru::*;
use crate::config::*;

// 根据问题,语言和题目信息评测,返回Task
pub fn evaluate(pro : & Problem, item : & Submission, lang : & Language, index : usize) {
    let fmt = "%Y-%m-%dT%H:%M:%S%.3fZ";
    // 进行评测
    // 更新状态为Running
    let mut lock = JOB_LIST.lock().unwrap();
    lock[index].state = State::Running;
    lock[index].result = Result::Running;
    drop(lock);
    // 创建临时文件夹
    let mut lock = TASK_NUM.lock().unwrap();
    let temp_dir = String::from(format!("temp_{}", lock));
    *lock += 1;
    drop(lock);
    match Command::new("mkdir")
    .arg(&temp_dir)
    .status() {
        Err(_) => {
            // 新建文件夹失败 系统故障
            let mut lock = JOB_LIST.lock().unwrap();
            lock[index].state = State::Finished;
            lock[index].result = Result::SystemError;
            for case in &mut lock[index].cases {
                case.result = Result::SystemError;
            }
            // let task = lock[index].clone();
            drop(lock);
            return;
        }
        Ok(_) => {}
    }
    
    // 写入源代码
    let mut source_file = File::create(format!("{}/{}", &temp_dir, lang.file_name)).unwrap();
    match source_file.write_all(item.source_code.as_bytes()) {
        Err(_) => {
            // 写入源代码失败 系统故障
            let mut lock = JOB_LIST.lock().unwrap();
            lock[index].state = State::Finished;
            lock[index].result = Result::SystemError;
            for case in &mut lock[index].cases {
                case.result = Result::SystemError;
            }
            // let task = lock[index].clone();
            drop(lock);
            return;
        }
        Ok(_) => {}
    }
    // 编译
    // 将编译测试点点状态更改为Running
    let mut lock = JOB_LIST.lock().unwrap();
    lock[index].cases[0].result = Result::Running;
    drop(lock);
    
    let mut commands = lang.command.clone();
    for c in &mut commands {
        if c == "%INPUT%" {
            *c = String::from(format!("{}/{}", &temp_dir,lang.file_name));
        } else if c == "%OUTPUT%" {
            *c = String::from(format!("{}/temp", &temp_dir));
        }
    }
    let mut is_compilation_error = false; // 编译是否出错
    let mut is_task_error = false; // 任务是否已有错误的测试点
    let compile_start_time = time::Instant::now(); // 计算编译时间开始
    let compile_state = Command::new(commands[0].clone())
    .args(&*&mut commands[1..])
    .status();
    let compile_end_time = time::Instant::now(); // 计算编译时间结束
    // 判断编译是否出错,出错则为系统故障
    let mut lock = JOB_LIST.lock().unwrap();
    match compile_state {
        Err(_) => {
            // 编译运行失败 系统故障
            lock[index].result = Result::SystemError;
            lock[index].state = State::Finished;
            drop(lock);
            return ;
        }
        _ => {}
    }
    match compile_state.unwrap().code().unwrap() {
        0 => {
            lock[index].cases[0].result = Result::CompilationSuccess;
        }
        1 => {
            lock[index].cases[0].result = Result::CompilationError;
            lock[index].result = Result::CompilationError;
            is_compilation_error = true;
            is_task_error = true;
        }
        _ => {

        }
    }
    lock[index].cases[0].time = (compile_end_time - compile_start_time).whole_microseconds() as u64;
    drop(lock);


    // 构建分组
    // 指定分组测评则按其分组
    // 否则每个测试点分一组
    let mut groups : Vec<Vec<u64>> = Vec::new();
    match &pro.misc {
        None => {}
        Some(misc) => {
            match &misc.packing {
                None => {
                    for i in 0..pro.cases.len() {
                        groups.push(Vec::from(vec![i as u64 +1]));
                    }
                }
                Some(g) => {
                    groups = g.clone();
                }
            }
        }
    }

    // 对每个组做测试
    for g in 0..groups.len() {
        // 若编译正确则进行分组测评,否则跳过测评直接返回默认测试点
        if !is_compilation_error {

            // 该组目前是否已有错误测试点
            let mut is_group_ok = true;

            // 该组目前得分
            let mut group_score : f64 = 0.0;

            // 对该组中的每个测试点
            for i in 0..groups[g].len() {
                let id = groups[g][i] as usize;
                // 若该组已有错误测试点,则将测试点结果置为skipped
                if !is_group_ok {
                    let mut lock = JOB_LIST.lock().unwrap();
                    lock[index].cases[id].result = Result::Skipped;
                    drop(lock);
                    continue;
                }
                // 若该组尚无错误测试点
                let mut is_case_ok = true; //该测试点是否出错
                // 将该测试点状态改为 Running
                let mut lock = JOB_LIST.lock().unwrap();
                lock[index].cases[id].result = Result::Running;
                drop(lock);
                // 设置重定向输入输出文件
                let in_file = File::open(format!("{}", pro.cases[id-1].input_file)).unwrap();
                let out_file = File::create(format!("{}/test_{}.out", &temp_dir, id-1)).unwrap();
                let start = time::Instant::now(); //程序运行计时开始
                // 运行可执行文件
                let run_state = Command::new(format!("./{}/temp", &temp_dir))
                .stdin(in_file)
                .stdout(out_file)
                .spawn();
                match run_state {
                    Err(_) => {
                        // 系统故障
                        is_case_ok = false;
                        let mut lock = JOB_LIST.lock().unwrap();
                        lock[index].cases[id].result = Result::SystemError;
                        is_group_ok = false;
                        //case.result = Result::SystemError;
                        if !is_task_error {
                            lock[index].result = Result::SystemError;
                            //task.result = Result::SystemError;
                            is_task_error = true;
                        }
                        drop(lock);
                    }
                    _ => {}
                }
                // 判断是否超时及运行时是否出错
                if is_case_ok {
                    //设置最大运行时间
                    let mut run_state = run_state.unwrap();
                    let max_duration = Duration::from_micros(pro.cases[id-1].time_limit + 100000);
                    let status_code = match run_state.wait_timeout(max_duration).unwrap() {
                        // 未超时
                        Some(status) => status.code(),
                        None => {
                            if pro.cases[id-1].time_limit != 0 {
                                // 限定时间且超时
                                run_state.kill().unwrap();
                            }
                            // 未限定时间 等待程序结束 
                            run_state.wait().unwrap().code()
                        }
                    };
                    let end = time::Instant::now(); //程序运行计时结束
                    let mut lock = JOB_LIST.lock().unwrap();
                    lock[index].cases[id].time = (end - start).whole_microseconds() as u64;
                    //case.time = (end - start).whole_microseconds() as u64;
                    match status_code {
                        None => {
                            // 超时
                            lock[index].cases[id].result = Result::TimeLimitExceeded;
                            is_group_ok = false;
                            //case.result = Result::TimeLimitExceeded;
                            is_case_ok = false;
                            if !is_task_error {
                                lock[index].result = Result::TimeLimitExceeded;
                                //task.result = Result::TimeLimitExceeded;
                                is_task_error = true;
                            }
                        }
                        _ => {
                            match  status_code.unwrap() {
                                0 => {
                                    // 未超时且运行无误
                                }
                                _ => {
                                    // 未超时且运行有误
                                    lock[index].cases[id].result = Result::RuntimeError;
                                    is_group_ok = false;
                                    is_case_ok = false;
                                    if !is_task_error {
                                        lock[index].result = Result::RuntimeError;
                                        is_task_error = true;
                                    }
                                }
                            }
                        }
                    }
                    drop(lock);
                }
                  
                // 判断文件是否一致
                if is_case_ok {
                    match pro.ty {
                        ProblemType::Standard => {
                            // 标准模式
                            // 将答案文件和输出文件按行读入
                            // 去除末尾空行
                            // 去除每行末尾的空格
                            // 逐行比较
                            let mut is_ok : bool = true;
                            let f_1 = File::open(format!("{}/test_{}.out", temp_dir, id-1)).unwrap();
                            let f_2 = File::open(pro.cases[id-1].answer_file.clone()).unwrap();
                            let br_1 = BufReader::new(f_1);
                            let br_2 = BufReader::new(f_2);
                            let mut lines_1 : Vec<String> = Vec::new();
                            let mut lines_2 : Vec<String> = Vec::new();

                            for line in br_1.lines() {
                                lines_1.push(line.unwrap());
                            }
                            for line in br_2.lines() {
                                lines_2.push(line.unwrap());
                            }
                            if lines_1.len() != lines_2.len() {
                                is_ok = false;
                            } else {
                                for i in 0..lines_2.len() {
                                    if lines_1[i].trim() != lines_2[i].trim() {
                                        is_ok = false;
                                        break;
                                    }
                                }
                            }
                            let mut lock = JOB_LIST.lock().unwrap();
                            if is_ok {
                                // 一致
                                lock[index].cases[id].result = Result::Accepted;
                                group_score += pro.cases[id-1].score;
                            } else {
                                // 不一致
                                lock[index].cases[id].result = Result::WrongAnswer;
                                is_group_ok = false;
                                if !is_task_error {
                                    lock[index].result = Result::WrongAnswer;
                                }
                            }
                            drop(lock);
                        }
                        ProblemType::Strict => {
                            // 严格模式
                            let mut is_ok : i32 = -1;
                            match Command::new("diff")
                            .arg(format!("{}/test_{}.out", temp_dir, id-1))
                            .arg(pro.cases[id-1].answer_file.clone())
                            .status(){
                                Err(_) => {
                                    // 系统故障
                                    is_case_ok = false;
                                    let mut lock = JOB_LIST.lock().unwrap();
                                    lock[index].cases[id].result = Result::SystemError;
                                    is_group_ok = false;
                                    if !is_task_error {
                                        lock[index].result = Result::SystemError;
                                        is_task_error = true;
                                    }
                                    drop(lock);
                                }
                                Ok(s) => {
                                    is_ok  = s.code().unwrap();
                                }
                            }
                            if is_case_ok {
                                let mut lock = JOB_LIST.lock().unwrap();
                                if is_ok == 0 {
                                    // 一致
                                    lock[index].cases[id].result = Result::Accepted;
                                    group_score += pro.cases[i].score;
                                } else {
                                    // 不一致
                                    lock[index].cases[id].result = Result::WrongAnswer;
                                    is_group_ok = false;
                                    if !is_task_error {
                                        lock[index].result = Result::WrongAnswer;
                                    }
                                }
                                drop(lock);
                            }
                        }
                        ProblemType::Spj => {
                            // 使用外部程序评测
                            match &pro.misc {
                                None => {
                                    // 系统错误
                                    //is_case_ok = false;
                                    let mut lock = JOB_LIST.lock().unwrap();
                                    lock[index].cases[id].result = Result::SystemError;
                                    is_group_ok = false;
                                    if !is_task_error {
                                        lock[index].result = Result::SystemError;
                                        is_task_error = true;
                                    }
                                    drop(lock);
                                }
                                Some(m) => {
                                    match &m.special_judge {
                                        None => {
                                            // 系统错误
                                            //is_case_ok = false;
                                            let mut lock = JOB_LIST.lock().unwrap();
                                            lock[index].cases[id].result = Result::SystemError;
                                            is_group_ok = false;
                                            //case.result = Result::SystemError;
                                            if !is_task_error {
                                                lock[index].result = Result::SystemError;
                                                //task.result = Result::SystemError;
                                                is_task_error = true;
                                            }
                                            drop(lock);
                                        }
                                        Some(s) => {
                                            // 重定向结果输出
                                            let out_file = File::create(format!("{}/outcome.txt", &temp_dir)).unwrap();
                                            let mut commands = s.clone();
                                            for i in 0..commands.len() {
                                                if commands[i] == "%ANSWER%" {
                                                    commands[i] = format!("{}", pro.cases[id-1].answer_file);
                                                } else if commands[i] == "%OUTPUT%" {
                                                    commands[i] = format!("{}/test_{}.out", temp_dir, id-1);
                                                }
                                            }
                                            // 运行测评程序
                                            match Command::new(&commands[0])
                                            .args(&commands[1..])
                                            .stdout(out_file)
                                            .status() {
                                                Err(_) => {
                                                    // Spj Error
                                                    let mut lock = JOB_LIST.lock().unwrap();
                                                    lock[index].cases[id].result = Result::SPJError;
                                                    is_group_ok = false;
                                                    if !is_task_error {
                                                        lock[index].result = Result::SPJError;
                                                        is_task_error = true;
                                                    }
                                                    drop(lock);
                                                }
                                                _ => {
                                                    // 正常测评
                                                    let f = File::open(format!("{}/outcome.txt", &temp_dir));
                                                    match f {
                                                        Err(_) => {
                                                            // Spj Error
                                                            let mut lock = JOB_LIST.lock().unwrap();
                                                            lock[index].cases[id].result = Result::SPJError;
                                                            is_group_ok = false;
                                                            if !is_task_error {
                                                                lock[index].result = Result::SPJError;
                                                                is_task_error = true;
                                                            }
                                                            drop(lock);
                                                        }
                                                        Ok(fi) => {
                                                            let br = BufReader::new(fi);
                                                            let mut out : Vec<String> = Vec::new();
                                                            for i in br.lines() {
                                                                out.push(i.unwrap());
                                                            }
                                                            // 若out的长度不为2,则为SPJ error
                                                            if out.len() != 2 {
                                                                let mut lock = JOB_LIST.lock().unwrap();
                                                                lock[index].cases[id].result = Result::SPJError;
                                                                is_group_ok = false;
                                                                if !is_task_error {
                                                                    lock[index].result = Result::SPJError;
                                                                    is_task_error = true;
                                                                }
                                                                drop(lock);
                                                            } else {
                                                                let mut lock = JOB_LIST.lock().unwrap();
                                                                if out[0] == "Accepted" {
                                                                    lock[index].cases[id].result = Result::Accepted;
                                                                    group_score += pro.cases[id-1].score;
                                                                } else {
                                                                    // 不一致
                                                                    lock[index].cases[id].result = Result::WrongAnswer;
                                                                    is_group_ok = false;
                                                                    if !is_task_error {
                                                                        lock[index].result = Result::WrongAnswer;
                                                                    }
                                                                }
                                                                lock[index].cases[id].info = out[1].clone();
                                                                drop(lock);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }  
                        }
                        _ => {
                        }
                    } 
                }
            }
            if is_group_ok {
                let mut lock = JOB_LIST.lock().unwrap();
                lock[index].score += group_score;
                drop(lock);
            }
        }
    }

    // 删除临时文件夹
    
    match std::fs::remove_dir_all(format!("{}", temp_dir)) {
        Err(_) => {
            // 系统故障 但已测评完毕
        }
        Ok(_) => {}
    }
    let mut lock = JOB_LIST.lock().unwrap();
    let now : DateTime<Utc> = Utc::now();
    lock[index].updated_time = now.format(fmt).to_string();
    lock[index].state = State::Finished;
    match &lock[index].result {
        Result::Running => {
            lock[index].result = Result::Accepted;
        }
        _ => {}
    }
    //let task = lock[index].clone();
    drop(lock);
    
    // 将JOB_LIST写入文件
    let lock = JOB_LIST.lock().unwrap();
    let job_list = lock.clone();
    drop(lock);
    let temp = serde_json::to_string_pretty(&job_list);
    let mut f = File::create("src/data/job.json").unwrap();
    match f.write_all(temp.unwrap().as_bytes()) {
        Err(_) => unimplemented!(),
        _ => {}
    }
}