use bench;
use stats::Fact;
use content_length::ContentLength;

#[derive(Clone)]
pub struct Engine {
    urls: Vec<String>,
    requests: usize,
    is_head: bool,
    kind: EngineKind,
}

#[derive(Clone)]
enum EngineKind {
    Reqwest,
    Hyper,
}

impl Engine {
    pub fn new(urls: Vec<String>, requests: usize) -> Engine {
        Engine {
            urls,
            requests,
            is_head: false,
            kind: EngineKind::Reqwest,
        }
    }

    pub fn set_to_head(mut self, is_head: bool) -> Self {
        self.is_head = is_head;
        self
    }

    pub fn with_hyper(mut self) -> Self {
        self.kind = EngineKind::Hyper;
        self
    }

    pub fn run<F>(self, f: F)
    where
        F: FnMut(Fact),
    {
        match &self.kind {
            &EngineKind::Reqwest => self.run_reqwest(f),
            &EngineKind::Hyper => self.run_hyper(f),
        };
    }

    fn run_reqwest<F>(&self, mut f: F)
    where
        F: FnMut(Fact),
    {
        use reqwest::{header, Client, Method, Request};
        let client = Client::new();

        let method = if self.is_head { Method::Head } else { Method::Get };

        for n in 0..self.requests {
            let url = &self.urls[n % self.urls.len()];

            let request = Request::new(method.clone(), url.parse().expect("Invalid url"));
            let (resp, duration) = bench::time_it(|| {
                let mut resp = client
                    .execute(request)
                    .expect("Failure to even connect is no good");
                let _ = resp.text();
                resp
            });
            let len = resp.headers()
                .get::<header::ContentLength>()
                .map(|len| **len)
                .unwrap_or(0);

            f(Fact::record(
                ContentLength::new(len),
                resp.status().as_u16(),
                duration,
            ));
        }
    }

    fn run_hyper<F>(&self, mut f: F)
    where
        F: FnMut(Fact),
    {
        use hyper::{header, Client, Method, Request, Uri};
        use hyper_tls::HttpsConnector;
        use tokio_core::reactor::Core;
        use futures::{Future, Stream};

        let mut core = Core::new().expect("Setting up tokio core failed");
        let handle = core.handle();
        let client = Client::configure()
            .connector(HttpsConnector::new(1, &handle).expect("To set up a http connector"))
            .build(&handle);

        let urls: Vec<Uri> = self.urls.iter().map(|url| url.parse().unwrap()).collect();

        let method = if self.is_head { Method::Head } else { Method::Get };

        for n in 0..self.requests {
            let uri = &urls[n % urls.len()];
            let request = client
                .request(Request::new(method.clone(), uri.clone()))
                .and_then(|response| {
                    let status = response.status().as_u16();
                    let content_length = response
                        .headers()
                        .get::<header::ContentLength>()
                        .map(|len| len.0)
                        .unwrap_or(0);
                    response
                        .body()
                        .concat2()
                        .map(move |_| (status, content_length))
                });
            let ((status, content_length), duration) =
                bench::time_it(|| core.run(request).expect("reactor run"));
            f(Fact::record(
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
        let eng = Engine::new(vec!["https://www.google.com".to_string()], 1);
        let mut fact: Option<Fact> = None;
        eng.run(|f| fact = Some(f));
        assert!(fact.is_some());
    }

    #[test]
    fn hyper_engine_can_collect_facts() {
        let eng = Engine::new(vec!["https://www.google.com".to_string()], 1).with_hyper();
        let mut fact: Option<Fact> = None;
        eng.run(|f| fact = Some(f));
        assert!(fact.is_some());
    }
}
