use serde::Serialize;

#[derive(Serialize)]
pub struct CheckResult {
    pub decision: String,
    pub rule: String,
    pub reason: String,
}

#[derive(Serialize, Clone)]
pub struct CheckStateResult {
    pub issues: Vec<StateIssue>,
}

#[derive(Serialize, Clone)]
pub struct StateIssue {
    pub severity: String,
    pub message: String,
}

pub fn print_json<T: Serialize>(value: &T) {
    println!("{}", serde_json::to_string(value).unwrap());
}

pub fn print_human(message: &str) {
    println!("{message}");
}
