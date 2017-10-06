extern crate hyper;
use api::MatchGetter;
use self::hyper::Error;

struct Match {
    id: i64,
    revision: Option<i32>
}

pub trait Loader {
    fn catchup(&self, steps: i16, matchGetter: &MatchGetter, history: &Vec<Match>) -> Result<Vec<Match>, Error>;
    fn process(&self, start: i64, steps: i16, matchGetter: &MatchGetter) -> Result<Vec<Match>, Error>;
}

struct AfaLoader {
    path: &'static str,
}

impl Loader for AfaLoader {
    fn catchup(&self, steps: i16, matchGetter: &MatchGetter, history: &Vec<Match>) -> Result<Vec<Match>, Error> {
        Ok(vec![])
    }
    fn process(&self, start: i64, steps: i16, matchGetter: &MatchGetter) -> Result<Vec<Match>, Error> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    extern crate hyper;
    extern crate serde_json;
    use api::MatchGetter;
    use self::hyper::Error;
    use self::serde_json::Value;

    struct FakeApi {}
    impl MatchGetter for FakeApi {
        fn get_match(&mut self, id: i64) -> Result<Value, Error> {
            Ok()
        }
    }
}
