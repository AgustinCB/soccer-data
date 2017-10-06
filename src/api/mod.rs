extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate tokio_core;
use std::io;
use self::futures::{Future, Stream};
use self::hyper::{Client, Error, Uri};
use self::hyper::error::UriError;
use self::hyper::client::HttpConnector;
use self::serde_json::Value;
use self::tokio_core::reactor::Core;

pub trait MatchGetter {
    fn get_match(&mut self, id: i64) -> Result<Value, Error>;
}

struct AfaApi {
    core: Core,
    client: Client<HttpConnector>
}

impl MatchGetter for AfaApi {
    fn get_match(&mut self, id: i64) -> Result<Value, Error> {
        let uri = get_match_uri(id)?;
        let future = self.client.get(uri).and_then(|res| {
            res.body().concat2()
        }).and_then(move |body| {
            Ok(serde_json::from_slice(&body).map_err(|e| {
                io::Error::new(
                    io::ErrorKind::Other,
                    e
                )
            })?)
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
}
