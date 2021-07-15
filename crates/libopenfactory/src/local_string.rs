#[derive(Debug, PartialEq, Eq)]
pub struct LocalString(String);

impl LocalString {
    pub fn from_str(s: &str) -> Self {
        LocalString(s.to_string())
    }
}