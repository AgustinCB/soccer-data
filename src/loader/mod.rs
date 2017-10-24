extern crate serde_json;
extern crate itertools;
use match_manager::Match;
use api::MatchGetter;
use self::itertools::unfold;
use self::serde_json::Value;
use std::cell::RefCell;

pub enum MatchInterval {
    AllRemaining,
    Next(i64)
}

pub trait Loader {
    fn catchup(&self, prevs: i64, history: &Vec<Match>) -> Result<Vec<Match>, String>;
    fn process(&self, start: i64, interval: MatchInterval) -> Result<Vec<Match>, String>;
}

struct AfaLoader {
    matches: RefCell<Box<MatchGetter>>,
    max_tries: i64
}

impl AfaLoader {
    pub fn new(matches: RefCell<Box<MatchGetter>>, max_tries: Option<i64>) -> AfaLoader {
        AfaLoader { matches: matches, max_tries: max_tries.unwrap_or(20) }
    }

    fn value_to_match(v: Value) -> Result<Match, String> {
        let id: i64 = v["match"]["matchId"].as_i64().ok_or("id not found")?;
        let revision: i64 = v["match"]["revision"].as_i64().ok_or("revision not found")?;
        let data = v.get("match").ok_or("Match not found")?;
        match revision {
            0 => Err(String::from("No revision!")),
            revision if revision > 0 => Ok(Match::new(id).data(data.clone())),
            revision => Err(String::from(format!("Invalid revision! {:?}", revision)))
        }
    }

    fn get_next_valid_match(&self, id: i64) -> Result<Match, String> {
        let mut api = self.matches.borrow_mut();
        let mut counter = 0;
        let mut match_attempt: Result<Match, String>;
        loop {
            let id_to_try = id + counter;
            match_attempt = api.get_match(id_to_try).and_then(AfaLoader::value_to_match);
            counter += 1;
            match match_attempt {
              Err(ref msg) if counter <= self.max_tries && *msg == String::from("Invalid status code: 404 Not Found") => continue,
              _ => break
            };
        };
        match_attempt
    }

    fn is_no_revision_error(match_attempt: &Result<Match, String>) -> bool {
        match *match_attempt {
            Err(ref msg) => *msg != String::from("No revision!"),
            _ => true
        }
    }

    fn process_interval(&self, from: i64, max_steps: Option<i64>) -> Result<Vec<Match>, String> {
        let mut steps = 0;
        unfold(Some(from), move |maybe_step| {
            if max_steps.map(|max| steps < max).unwrap_or(true) {
                maybe_step.map(|step| {
                    let res = self.get_next_valid_match(step);
                    *maybe_step = res.as_ref().map(|result| result.id+1).ok();
                    steps += 1;
                    res
                })
            } else {
                None
            }
        })
        .filter(AfaLoader::is_no_revision_error)
        .collect::<Result<Vec<_>, _>>()
    }
}

impl Loader for AfaLoader {
    fn catchup(&self, steps: i64, history: &Vec<Match>) -> Result<Vec<Match>, String> {
        let initial_step = match history.get(history.len() - (steps as usize)) {
            None => history.last().map(|m| m.id + 1).unwrap_or(1),
            Some(v) => v.id,
        };
        self.process_interval(initial_step, None)
    }
    fn process(&self, start: i64, interval: MatchInterval) -> Result<Vec<Match>, String> {
        match interval {
            MatchInterval::AllRemaining => self.catchup(0, &vec![Match::new(start-1)]),
            MatchInterval::Next(i) => self.process_interval(start, Some(i))
        }
    }
}

#[cfg(test)]
mod tests {
    extern crate serde_json;
    extern crate hyper;
    use api::MatchGetter;
    use super::{AfaLoader, Loader, Match, MatchInterval};
    use self::hyper::StatusCode;
    use self::serde_json::Value;
    use std::cell::RefCell;

    struct FakeAfaApi {}

    impl MatchGetter for FakeAfaApi {
        fn get_match(&mut self, id: i64) -> Result<Value, String> {
            match id {
                1 => Ok(json!({
                    "match": {
                        "revision": 1,
                        "matchId": 1
                    }
                })),
                2 => Ok(json!({
                    "match": {
                        "revision": 2,
                        "matchId": 2
                    }
                })),
                3 => Ok(json!({
                    "match": {
                        "revision": 3,
                        "matchId": 3
                    }
                })),
                4 => Err(format!("Invalid status code: {}", StatusCode::NotFound)),
                5 => Err(format!("Invalid status code: {}", StatusCode::NotFound)),
                6 => Err(format!("Invalid status code: {}", StatusCode::NotFound)),
                7 => Ok(json!({
                    "match": {
                        "revision": 4,
                        "matchId": 7
                    }
                })),
                8 => Ok(json!({
                    "match": {
                        "revision": 5,
                        "matchId": 8
                    }
                })),
                n => Ok(json!({
                    "match": {
                        "revision": 0,
                        "matchId": n
                    }
                }))
            }
        }
    }

    fn create_data(id: i64, revision: i64) -> Value {
        json!({"matchId": id, "revision": revision})
    }

    fn get_result_from_catchup(prevs: i64, history: &Vec<Match>) -> Vec<Match> {
        let api = FakeAfaApi{};
        let loader = AfaLoader::new(RefCell::new(Box::new(api)), None);
        loader.catchup(prevs, history).expect("Error getting matches")
    }
    
    fn get_result_from_process(start: i64, interval: MatchInterval, max_tries: Option<i64>) -> Vec<Match> {
        let api = FakeAfaApi{};
        let loader = AfaLoader::new(RefCell::new(Box::new(api)), max_tries);
        loader.process(start, interval).expect("Error getting matches")
    }

    #[test]
    fn in_catchup_it_should_get_all_when_empty_history() {
        let res = get_result_from_catchup(0, &vec![]);
        assert_eq!(res, [Match::new(1).data(create_data(1, 1)), Match::new(2).data(create_data(2, 2)), Match::new(3).data(create_data(3, 3)), Match::new(7).data(create_data(7, 4)), Match::new(8).data(create_data(8, 5))])
    }

    #[test]
    fn in_catchup_it_should_get_remaining_when_there_is_history_and_zero_steps_back() {
        let res = get_result_from_catchup(0, &vec![Match::new(1)]);
        assert_eq!(res, [Match::new(2).data(create_data(2, 2)), Match::new(3).data(create_data(3, 3)), Match::new(7).data(create_data(7, 4)), Match::new(8).data(create_data(8, 5))])
    }

    #[test]
    fn in_catchup_it_should_get_remaining_and_steps_back_when_there_is_history_and_steps() {
        let res = get_result_from_catchup(2, &vec![Match::new(1), Match::new(2), Match::new(3)]);
        assert_eq!(res, [Match::new(2).data(create_data(2, 2)), Match::new(3).data(create_data(3, 3)), Match::new(7).data(create_data(7, 4)), Match::new(8).data(create_data(8, 5))])
    }

    #[test]
    fn in_catchup_it_should_get_new_remaining_and_steps_back_when_there_is_history_and_steps() {
        let res = get_result_from_catchup(2, &vec![Match::new(1), Match::new(2), Match::new(3)]);
        assert_eq!(res, [Match::new(2).data(create_data(2, 2)), Match::new(3).data(create_data(3, 3)), Match::new(7).data(create_data(7, 4)), Match::new(8).data(create_data(8, 5))])
    }

    #[test]
    fn in_catchup_it_should_get_nothing_when_you_are_at_the_end_of_history() {
        let res = get_result_from_catchup(0, &vec![Match::new(8)]);
        assert_eq!(res, [])
    }

    #[test]
    fn in_process_it_should_get_next_when_interval_is_available() {
        let res = get_result_from_process(2, MatchInterval::Next(3), None);
        assert_eq!(res, [Match::new(2).data(create_data(2, 2)), Match::new(3).data(create_data(3, 3)), Match::new(7).data(create_data(7, 4))])
    }

    #[test]
    fn in_process_it_should_get_next_when_interval_is_available_and_there_re_missing() {
        let res = get_result_from_process(3, MatchInterval::Next(3), None);
        assert_eq!(res, [Match::new(3).data(create_data(3, 3)), Match::new(7).data(create_data(7, 4)), Match::new(8).data(create_data(8, 5))])
    }

    #[test]
    fn in_process_it_should_get_remaining_when_interval_is_too_big() {
        let res = get_result_from_process(1, MatchInterval::Next(10), None);
        assert_eq!(res, [Match::new(1).data(create_data(1, 1)), Match::new(2).data(create_data(2, 2)), Match::new(3).data(create_data(3, 3)), Match::new(7).data(create_data(7, 4)), Match::new(8).data(create_data(8, 5))])
    }

    #[test]
    fn in_process_it_should_get_none_when_start_is_last() {
        let res = get_result_from_process(9, MatchInterval::Next(10), None);
        assert_eq!(res, [])
    }

    #[test]
    fn in_process_it_should_get_all_remaining() {
        let res = get_result_from_process(1, MatchInterval::AllRemaining, None);
        assert_eq!(res, [Match::new(1).data(create_data(1, 1)), Match::new(2).data(create_data(2, 2)), Match::new(3).data(create_data(3, 3)), Match::new(7).data(create_data(7, 4)), Match::new(8).data(create_data(8, 5))])
    }

    #[should_panic(expected = "Error getting matches: \"Invalid status code: 404 Not Found\"")]
    #[test]
    fn in_process_it_should_fail_when_passing_no_steps() {
        get_result_from_process(1, MatchInterval::Next(10), Some(0));
    }
}
