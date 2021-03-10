use chrono::NaiveDateTime;

#[derive(Debug, Eq, PartialEq)]
pub struct Photo {
    pub date: NaiveDateTime,
    pub path: String,
    pub new_path: Option<String>,
}