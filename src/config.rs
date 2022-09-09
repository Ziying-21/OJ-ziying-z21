use serde_derive::{self, Serialize, Deserialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Server {
    pub bind_address : Option<String>,
    pub bind_port : Option<u16>
}


#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(rename_all = "snake_case")]
pub enum ProblemType {
    Standard,
    Strict,
    Spj,
    DynamicRanking
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProblemCase {
    pub score : f64,
    pub input_file : String,
    pub answer_file : String,
    pub time_limit : u64,
    pub memory_limit : u64
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Misc {
    pub packing : Option<Vec<Vec<u64>>>,
    pub special_judge : Option<Vec<String>>,
    pub dynamic_ranking_ratio : Option<f64>
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Problem {
    pub id : u64,
    pub name : String,
    #[serde(rename = "type")]
    pub ty : ProblemType,
    pub misc : Option<Misc>,
    pub cases : Vec<ProblemCase>,
}

impl Problem {
    pub fn new() -> Problem {
        Problem{
            id : 0,
            name : String::new(),
            ty : ProblemType::Standard,
            misc : None,
            cases : Vec::new()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Language {
    pub name : String,
    pub file_name : String,
    pub command : Vec<String>
}

impl Language {
    pub fn new() -> Language {
        Language { 
            name: String::new(),
            file_name: String::new(), 
            command: Vec::new() 
        }
    }
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub server : Server,
    pub problems : Vec<Problem>,
    pub languages : Vec<Language>
}