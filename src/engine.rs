use bench;
use content_length::ContentLength;
use stats::Fact;

/// The engine of making requests. The engine implements making the requests and producing
/// facts for the stats collector to process.
#[derive(Clone)]
pub struct Engine {
    urls: Vec<String>,
    method: Method,
    kind: Kind,
}

/// The methods that are supported by the current implementations. These are currently
/// body-less methods so that we don't need to load up any additional content.
#[derive(Clone, Copy)]
pub enum Method {
    Get,
    Head,
}
const DEFAULT_METHOD: Method = Method::Get;

#[derive(Clone, Copy)]
enum Kind {
    Reqwest,
    Hyper,
}
const DEFAULT_KIND: Kind = Kind::Reqwest;

impl Engine {
    /// Creates a new engine. The engine will default to using `reqwest`
    pub fn new(urls: Vec<String>) -> Engine {
        Engine {
            urls,
            method: DEFAULT_METHOD,
            kind: DEFAULT_KIND,
        }
    }

    /// Sets the method to use with the requests
    pub fn with_method(mut self, method: Method) -> Self {
        self.method = method;
        self
    }

    /// Sets the engine to be a hyper engine
    pub fn with_hyper(mut self) -> Self {
        self.kind = Kind::Hyper;
        self
    }

    /// Consumes self to start up the engine and begins making requests. It will callback
    /// to the collector to allow the caller to capture requests.
    pub fn run<F>(self, requests: usize, collect: F)
    where
        F: FnMut(Fact),
    {
        match self.kind {
            Kind::Reqwest => self.run_reqwest(requests, collect),
            Kind::Hyper => self.run_hyper(requests, collect),
        };
    }

    fn run_reqwest<F>(&self, requests: usize, mut collect: F)
    where
        F: FnMut(Fact),
    {
        use reqwest::{self, Client, Request};
        let client = Client::new();

        let method = match self.method {
            Method::Get => reqwest::Method::Get,
            Method::Head => reqwest::Method::Head,
        };

        for n in 0..requests {
            let url = &self.urls[n % self.urls.len()];

            let request = Request::new(method.clone(), url.parse().expect("Invalid url"));
            let mut len = 0;
            let (resp, duration) = bench::time_it(|| {
                let mut resp = client
                    .execute(request)
                    .expect("Failure to even connect is no good");
                if let Ok(body) = resp.text() {
                    len = body.len();
                }
                resp
            });

            collect(Fact::record(
                ContentLength::new(len as u64),
                resp.status().as_u16(),
                duration,
            ));
        }
    }

    fn run_hyper<F>(&self, requests: usize, mut collect: F)
    where
        F: FnMut(Fact),
    {
        use futures::{Future, Stream};
        use hyper::{self, Client, Request, Uri};
        use hyper_tls::HttpsConnector;
        use tokio_core::reactor::Core;

        let mut core = Core::new().expect("Setting up tokio core failed");
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(1, &handle).expect("To set up a http connector"))
            .build(&handle);

        let urls: Vec<Uri> = self.urls.iter().map(|url| url.parse().unwrap()).collect();

        let method = match self.method {
            Method::Get => hyper::Method::Get,
            Method::Head => hyper::Method::Head,
        };

        for n in 0..requests {
            let uri = &urls[n % urls.len()];
            let request = client
                .request(Request::new(method.clone(), uri.clone()))
                .and_then(|response| {
                    let status = response.status().as_u16();
                    response
                        .body()
                        .concat2()
                        .map(move |body| (status, body.len() as u64))
                });
            let ((status, content_length), duration) =
                bench::time_it(|| core.run(request).expect("reactor run"));
            collect(Fact::record(
                ContentLength::new(content_length),
                status,
                duration,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reqwest_engine_can_collect_facts() {
        let eng = Engine::new(vec!["https://www.google.com".to_string()]);
        let mut fact: Option<Fact> = None;
        eng.run(1, |f| fact = Some(f));
        assert!(fact.is_some());
    }

    #[test]
    fn hyper_engine_can_collect_facts() {
        let eng = Engine::new(vec!["https://www.google.com".to_string()]).with_hyper();
        let mut fact: Option<Fact> = None;
        eng.run(1, |f| fact = Some(f));
        assert!(fact.is_some());
    }
}
