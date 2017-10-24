extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate tokio_core;
use self::futures::{Future, Stream};
use self::hyper::{Client, StatusCode, Uri};
use self::hyper::error::UriError;
use self::hyper::client::HttpConnector;
use self::serde_json::Value;
use self::tokio_core::reactor::Core;

pub trait MatchGetter {
    // TODO: Use enum error instead of string
    fn get_match(&mut self, id: i64) -> Result<Value, String>;
}

struct AfaApi {
    core: Core,
    client: Client<HttpConnector>
}

impl MatchGetter for AfaApi {
    fn get_match(&mut self, id: i64) -> Result<Value, String> {
        let uri = get_match_uri(id).map_err(|e| {
            format!("Error parsing url: {}", e)
        })?;
        let future = self.client.get(uri)
            .map_err(|err| {
                format!("Error getting match {}: {}", id, err)
            })
            .and_then(|res| {
                match res.status() {
                    StatusCode::Ok => Ok(res.body()),
                    code => Err(format!("Invalid status code: {}", code))
                }
            })
            .and_then(|body| {
                body.concat2().map_err(|err| {
                    format!("Error concatenating body: {}", err)
                })
            })
            .and_then(|body| {
                serde_json::from_slice(&body).map_err(|e| {
                    format!("Error parsing body: {}", e)
                })
            });
        self.core.run(future)
    }
}

fn get_match_uri(id: i64) -> Result<Uri, UriError> {
    format!("http://www.afa.org.ar/deposito/html/v3/htmlCenter/data/deportes/futbol/primeraa/events/{}.json", id)
        .parse()
}

#[cfg(test)]
mod tests {
    extern crate hyper;
    extern crate tokio_core;
    use self::hyper::Client;
    use self::tokio_core::reactor::Core;
    use super::MatchGetter;

    #[test]
    fn it_gets_a_match() {
        let core = Core::new().expect("Error creating core");
        let client = Client::new(&core.handle());
        let mut api = super::AfaApi { core: core, client: client };
        let v = api.get_match(371133).expect("Error getting the match");
        assert_eq!(v["Revision"], "$Revision: 1318 $");
    }

    #[test]
    fn it_fails_when_404() {
        let core = Core::new().expect("Error creating core");
        let client = Client::new(&core.handle());
        let mut api = super::AfaApi { core: core, client: client };
        let e = api.get_match(-1).unwrap_err();
        assert_eq!(e, "Invalid status code: 404 Not Found");
    }
}
