use actix_web::{HttpResponse, Responder, get, web};
use crate::{stru::*, USER_LIST, JOB_LIST, config::{ Problem}, CONTEST_LIST};

// 返回arr中第一个num的下标, 找不到则返回-1
fn find(arr : &Vec<u64> , num : u64) -> i32 {
    let mut index = -1;
    for i in 0..arr.len() {
        if arr[i] == num {
            index = i as i32;
            break;
        }
    }
    index
}


#[get("/contests/{contestid}/ranklist")]
pub async fn get_ranklist(contestid : web::Path<usize>, item : web::Query<RankStandard>, config: web::Data<NewConfig>) -> impl Responder {
    
    // 加载比赛信息
    let contest_id = contestid.into_inner();
    let lock = CONTEST_LIST.lock().unwrap();
    if contest_id > lock.len() {
        drop(lock);
        return HttpResponse::NotFound().json(
            Error{
                code : 3,
                reason : String::from("ERR_NOT_FOUND"),
                message : String::from(format!("Contest {} not found.", contest_id))
            }
        );
    }
    let mut contest = Contest::new();
    if contest_id != 0 {
        contest = lock[contest_id-1].clone();
    }
    drop(lock);
    // 加载排序标准
    let mut standard = RankStandard::new();
    match &item.scoring_rule {
        None => {}
        Some(s) => {
            standard.scoring_rule = Some(s.clone());
        }
    }
    match &item.tie_breaker {
        None => {}
        Some(t) => {
            standard.tie_breaker = Some(t.clone());
        }
    }
    // 根据比赛信息加载要用到的job和problem
    let lock = JOB_LIST.lock().unwrap();
    let job_list = lock.clone();
    drop(lock);

    // 比赛要用到的job
    let mut all_task : Vec<Task> = Vec::new();
    for job in job_list {
        if contest_id == 0 {
            // 比赛id为0
            // JOB_LIST中在config中有对应题目的JOB符合要求
            let mut is_ok = false;
            for problem in &config.config.problems {
                if problem.id == job.submission.problem_id {
                    is_ok = true;
                    break;
                }
            }
            if is_ok {
                all_task.push(job);
            }
            continue;
        }
        if job.submission.contest_id == contest_id as u64 {
            all_task.push(job);
        }
    }
    // 比赛要用到的problem
    let mut all_problem : Vec<Problem> = Vec::new();
    for pro in &config.config.problems {
        if contest_id == 0 {
            all_problem.push(pro.clone());
            continue;
        }
        if contest.problem_ids.contains(&pro.id) {
            all_problem.push(pro.clone());
        }
    }
    // 比赛中的User
    let lock = USER_LIST.lock().unwrap();
    let user_list_len = match contest_id {
        0 => {
            lock.len()
        }
        _ => {
            contest.user_ids.len()
        }
    };
    drop(lock);
    let mut temp_user_ranked = Vec::from(vec![RankedUserTemp::new(); user_list_len]);
    
    for i in 0..temp_user_ranked.len() {
        if contest_id == 0 {
            temp_user_ranked[i].user.id = Some(i as u64);
        } else {
            temp_user_ranked[i].user.id = Some(contest.user_ids[i]);
        } 
    }

    // 遍历任务列表中的所有任务
    for task in all_task {
        // 判断该任务是否应该是这位用户用来记分的任务,如果是则替换,否则不替换
        let user_id = task.submission.user_id;
        // 获取该用户的下标
        let user_index = if contest_id == 0 {
            user_id as usize
        } else {
            find(&contest.user_ids, user_id) as usize
        };
        // 该用户的总提交次数加一
        temp_user_ranked[user_index].total_sub += 1;
        let mut is_pro_exist = false;
        for i in 0..temp_user_ranked[user_index].counted_task.len() {
            // 同一个题目
            if temp_user_ranked[user_index].counted_task[i].submission.problem_id == task.submission.problem_id {
                match standard.scoring_rule {
                    None => {} // 不可能发生
                    Some(ScoringRule::Highest) => {
                        // 按照分数最高
                        if task.score > temp_user_ranked[user_index].counted_task[i].score {
                            temp_user_ranked[user_index].counted_task[i] = task.clone();
                            if task.created_time > temp_user_ranked[user_index].latest_sub {
                                temp_user_ranked[user_index].latest_sub = task.created_time.clone();
                            }
                        }
                    }
                    Some(ScoringRule::Latest) => {
                        // 按照提交最晚
                        if task.created_time > temp_user_ranked[user_index].counted_task[i].created_time {
                            temp_user_ranked[user_index].counted_task[i] = task.clone();
                            if task.created_time > temp_user_ranked[user_index].latest_sub {
                                temp_user_ranked[user_index].latest_sub = task.created_time.clone();
                            }
                        }
                    }
                }
                is_pro_exist = true;
                break;
            }
        }
        if !is_pro_exist {
            // 尚未遍历到该problem对应的job
            temp_user_ranked[user_index].counted_task.push(task.clone());
            if task.created_time > temp_user_ranked[user_index].latest_sub {
                temp_user_ranked[user_index].latest_sub = task.created_time.clone();
            }
        }
    }
    // 对每个用户计算最终得分
    for user in &mut temp_user_ranked {
        for task in &user.counted_task {
            user.total_score += task.score;
        }
    }

    // 对每个用户判断是否没有提交
    for user in &mut temp_user_ranked {
        if user.counted_task.len() == 0{
            // 若没有提交则把时间设置为无穷晚
            user.latest_sub = String::from("9999-12-31T23:59:59.000Z");
        }
    }

    // 为每个用户补齐所有题目
    for user in &mut temp_user_ranked {
        for problem in &all_problem {
            let mut is_exist = false;
            for task in &user.counted_task {
                if task.submission.problem_id == problem.id {
                    is_exist = true;
                    break;
                }
            }
            if !is_exist {
                let mut new_task = Task::new();
                new_task.submission.problem_id = problem.id;
                user.counted_task.push(new_task);
            }
        }
    }
    // 对每个用户将任务按照problemID排序
    for user in &mut temp_user_ranked {
        for i in 0..user.counted_task.len() {
            for j in 0..(user.counted_task.len() - i - 1) {
                if contest_id != 0 {
                    // 比赛id为0
                    let j_index = find(&contest.problem_ids, user.counted_task[j].submission.problem_id);
                    let j_post_index = find(&contest.problem_ids, user.counted_task[j+1].submission.problem_id);
                    if j_index > j_post_index {
                        let temp_task = user.counted_task[j].clone();
                        user.counted_task[j] = user.counted_task[j+1].clone();
                        user.counted_task[j+1] = temp_task;
                    }
                    continue;
                }
                // 比赛id不为0
                let mut j_index : usize = 0;
                let mut j_post_index : usize = 0;
                for k in 0..config.config.problems.len() {
                    if user.counted_task[j].submission.problem_id == config.config.problems[k].id {
                        j_index = k;
                        break;
                    }
                }
                for k in 0..config.config.problems.len() {
                    if user.counted_task[j+1].submission.problem_id == config.config.problems[k].id {
                        j_post_index = k;
                        break;
                    }
                }
                if j_index > j_post_index {
                    let temp_task = user.counted_task[j].clone();
                    user.counted_task[j] = user.counted_task[j+1].clone();
                    user.counted_task[j+1] = temp_task;
                }
            }
        }
    }
    // 按照排序标准排序
    for i in 0..temp_user_ranked.len() {
        for j in 0..(temp_user_ranked.len() - i - 1) {
            if temp_user_ranked[j].total_score < temp_user_ranked[j+1].total_score {
                let temp = temp_user_ranked[j].clone();
                temp_user_ranked[j] = temp_user_ranked[j+1].clone();
                temp_user_ranked[j+1] = temp;
            } else if temp_user_ranked[j].total_score == temp_user_ranked[j+1].total_score {
                match standard.tie_breaker {
                    Some(TieBreaker::SubmissionCount) => {
                        if temp_user_ranked[j].total_sub > temp_user_ranked[j+1].total_sub {
                            let temp = temp_user_ranked[j].clone();
                            temp_user_ranked[j] = temp_user_ranked[j+1].clone();
                            temp_user_ranked[j+1] = temp;
                        }
                    }
                    Some(TieBreaker::SubmissionTime) => {
                        if temp_user_ranked[j].latest_sub > temp_user_ranked[j+1].latest_sub {
                            let temp = temp_user_ranked[j].clone();
                            temp_user_ranked[j] = temp_user_ranked[j+1].clone();
                            temp_user_ranked[j+1] = temp;
                        }
                    }
                    Some(TieBreaker::UserId) => {
                        if temp_user_ranked[j].user.id > temp_user_ranked[j+1].user.id {
                            let temp = temp_user_ranked[j].clone();
                            temp_user_ranked[j] = temp_user_ranked[j+1].clone();
                            temp_user_ranked[j+1] = temp;
                        }
                    }
                    _ => {
                        if temp_user_ranked[j].user.id > temp_user_ranked[j+1].user.id {
                            let temp = temp_user_ranked[j].clone();
                            temp_user_ranked[j] = temp_user_ranked[j+1].clone();
                            temp_user_ranked[j+1] = temp;
                        }
                    }
                }
            }
        }
    }
    // 给出名次
    for i in 0..temp_user_ranked.len() {
        if i == 0 {
            temp_user_ranked[i].rank = 1;
            continue;
        }
        if temp_user_ranked[i].total_score < temp_user_ranked[i-1].total_score {
            temp_user_ranked[i].rank = i as u64+1;
            continue;
        }
        match standard.tie_breaker {
            Some(TieBreaker::SubmissionTime) => {
                if temp_user_ranked[i].latest_sub == temp_user_ranked[i-1].latest_sub {
                    temp_user_ranked[i].rank = temp_user_ranked[i-1].rank;
                } else {
                    temp_user_ranked[i].rank = i as u64 +1;
                }
            }
            Some(TieBreaker::SubmissionCount) => {
                if temp_user_ranked[i].total_sub == temp_user_ranked[i-1].total_sub {
                    temp_user_ranked[i].rank = temp_user_ranked[i-1].rank;
                } else {
                    temp_user_ranked[i].rank = i as u64 +1;
                }
            }
            Some(TieBreaker::UserId) => {
                temp_user_ranked[i].rank = i as u64 + 1;
            }
            None => {
                if temp_user_ranked[i].total_score == temp_user_ranked[i-1].total_score {
                    temp_user_ranked[i].rank = temp_user_ranked[i-1].rank;
                }
            }
        }
    }
    // 给出排行榜
    let mut ranked_user_list : Vec<RankedUser> = Vec::new();
    for u in temp_user_ranked {
        let mut new_ranked_user = RankedUser {
            user : User::new(),
            rank : 0,
            scores : Vec::new()
        };
        let lock = USER_LIST.lock().unwrap();
        new_ranked_user.user.id = lock[u.user.id.unwrap() as usize].id;
        new_ranked_user.user.name = lock[u.user.id.unwrap() as usize].name.clone();
        new_ranked_user.rank = u.rank;
        for task in u.counted_task {
            new_ranked_user.scores.push(task.score);
        }
        ranked_user_list.push(new_ranked_user);
    }
    return HttpResponse::Ok().json(ranked_user_list);

//HttpResponse::Ok().json(standard)
}