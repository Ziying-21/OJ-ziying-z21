// use std::sync::mpsc::Sender;

// use std::ops::Sub;
use crate::config::*;
use serde_derive::{Serialize, Deserialize};
use crossbeam;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum State {
    Queueing,
    Running,
    Finished,
    Canceled
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Result {
    Waiting,
    Running,
    Accepted,
    #[serde(rename = "Compilation Error")]
    CompilationError,
    #[serde(rename = "Compilation Success")]
    CompilationSuccess,
    #[serde(rename = "Wrong Answer")]
    WrongAnswer,
    #[serde(rename = "Runtime Error")]
    RuntimeError,
    #[serde(rename = "Time Limit Exceeded")]
    TimeLimitExceeded,
    #[serde(rename = "Memory Limit Exceeded")]
    MemoryLimitExceeded,
    #[serde(rename = "System Error")]
    SystemError,
    #[serde(rename = "SPJ Error")]
    SPJError,
    Skipped
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct T {
    pub num : i32
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MyObj {
    pub name: String,
    pub number: i32,
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Error {
    pub code : u64,
    pub reason : String,
    pub message : String
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Case {
    pub id: u64,
    pub result: Result,
    pub time: u64,
    pub memory: u64,
    pub info: String
}

impl Case {
    pub fn new() -> Case {
        Case { 
            id: 0, 
            result: Result::Waiting, 
            time: 0, 
            memory: 0, 
            info: String::new() 
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Submission {
    pub source_code : String,
    pub language : String,
    pub user_id : u64,
    pub contest_id: u64,
    pub problem_id: u64
}

impl Submission {
    pub fn new() -> Submission {
        Submission { 
            source_code: String::new(), 
            language: String::new(), 
            user_id: 0, 
            contest_id: 0, 
            problem_id: 0
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Task {
    pub id : usize,
    pub created_time : String,
    pub updated_time : String,
    pub submission : Submission,
    pub state : State,
    pub result : Result,
    pub score : f64,
    pub cases : Vec<Case>,
}

impl Task {
    pub fn new() -> Task {
        Task {
            id : 0,
            created_time : String::new(),
            updated_time : String::new(),
            submission : Submission::new(),
            state : State::Queueing,
            result : Result::Waiting,
            score : 0.0,
            cases : Vec::new()
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct User {
    pub id : Option<u64>,
    pub name : String
}

impl User {
    pub fn new() -> Self {
        User { 
            id: None, 
            name: String::new() 
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum ScoringRule {
    Latest,
    Highest
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "snake_case")]
pub enum TieBreaker {
    SubmissionTime,
    SubmissionCount,
    UserId
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RankStandard {
    pub scoring_rule : Option<ScoringRule>,
    pub tie_breaker : Option<TieBreaker>
}

impl  RankStandard {
    pub fn new() -> Self {
        RankStandard { 
            scoring_rule: Some(ScoringRule::Latest), 
            tie_breaker: None
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RankedUser {
    pub user : User,
    pub rank : u64,
    pub scores : Vec<f64>
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct RankedUserTemp {
    pub user : User,
    pub rank : u64,
    pub total_score : f64,
    pub submission_time : u64,
    pub counted_task : Vec<Task>,
    pub latest_sub : String,
    pub total_sub : u64
}

impl RankedUserTemp {
    pub fn new() -> Self {
        RankedUserTemp { 
            user: User::new(), 
            rank: 0, 
            total_score : 0.0,
            submission_time: 0, 
            counted_task: Vec::new(),
            latest_sub : String::from("1970-01-01T00:00:00.000Z"),
            total_sub : 0
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Contest {
    pub id : Option<u64>,
    pub name : String,
    pub from : String,
    pub to : String,
    pub problem_ids : Vec<u64>,
    pub user_ids : Vec<u64>,
    pub submission_limit : u64
}

impl Contest {
    pub fn new() -> Self {
        Contest { 
            id: None, 
            name: String::new(), 
            from: String::new(), 
            to: String::new(), 
            problem_ids: Vec::new(), 
            user_ids: Vec::new(), 
            submission_limit: 0
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct EvaluatePara {
    pub language : Language,
    pub submission : Submission,
    pub problem : Problem,
    pub index : usize
}

#[derive(Clone)]
pub struct NewConfig {
    pub config : Config,
    pub sender : Option<crossbeam::channel::Sender<EvaluatePara>>
}
// crossbeam::crossbeam_channel::Sender