extern crate futures;
extern crate hyper;
extern crate serde_json;
extern crate tokio_core;
use std::io;
use self::futures::{Future, Stream};
use self::hyper::{Client, Error, Uri};
use self::hyper::client::HttpConnector;
use self::serde_json::Value;
use self::tokio_core::reactor::Core;

struct AfaApi {
    core: Core,
    client: Client<HttpConnector>
}

impl AfaApi {
    pub fn get_match(&mut self, id: i64) -> Result<Value, Error> {
        let future = self.client.get(get_match_uri(id)).and_then(|res| {
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

fn get_match_uri(id: i64) -> Uri {
    format!("http://www.afa.org.ar/deposito/html/v3/htmlCenter/data/deportes/futbol/primeraa/events/{}.json", id)
        .parse()
        .expect("Failed to parse string")
}

#[cfg(test)]
mod tests {
    extern crate hyper;
    extern crate tokio_core;
    use self::hyper::Client;
    use self::tokio_core::reactor::Core;

    #[test]
    fn it_gets_a_match() {
        let core = Core::new().expect("Error creating core");
        let client = Client::new(&core.handle());
        let mut api = super::AfaApi { core: core, client: client };
        let v = api.get_match(371133).expect("Error getting the match");
        assert_eq!(v["Revision"], "$Revision: 1318 $");
    }
}
