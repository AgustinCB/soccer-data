extern crate serde_json;
use self::serde_json::Value;
use std::fmt;

#[derive(Debug, PartialEq)]
pub struct Match {
    pub id: i64,
    data: Value,
}

impl Match {
    pub fn new(id: i64) -> Match {
        Match { id: id, data: json!({}) }
    }
    pub fn data(mut self, data: Value) -> Self {
        self.data = data;
        self
    }
}

pub struct MatchManager {
    path: &'static str
}

impl MatchManager {
    pub fn new(path: &'static str) -> MatchManager {
        MatchManager { path: path }
    }

    pub fn history(self) -> Vec<Match> {
        vec![]
    }

    pub fn save(self, matches: Vec<Match>) -> fmt::Result {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;
    use self::serde_json::Value;
    use super::MatchManager;

    #[test]
    fn it_gets_matchs_in_the_path() {
    }

    #[test]
    fn it_gets_no_match_in_empty_path() {
    }

    #[test]
    fn it_saves_matches_in_path() {
    }
}
