use bench;
use stats::Fact;

pub struct Engine {
    urls: Vec<String>,
    requests: usize,
    kind: EngineKind,
}

enum EngineKind {
    Reqwest,
}

impl Engine {
    pub fn new(urls: Vec<String>, requests: usize) -> Engine {
        Engine {
            urls,
            requests,
            kind: EngineKind::Reqwest
        }
    }

    pub fn run<F>(self, f: F) where F: FnMut(Fact) {
        match &self.kind {
            &EngineKind::Reqwest => self.run_reqwest(f),
        };
    }

    fn run_reqwest<F>(&self, mut f: F) where F: FnMut(Fact) {
        use reqwest::{Client, Method, Request};
        let client = Client::new();

        for n in 0..self.requests {
            let url = &self.urls[n % self.urls.len()];

            let request = Request::new(Method::Get, url.parse().expect("Invalid url"));
            let (resp, duration) = bench::time_it(|| {
                let mut resp = client
                    .execute(request)
                    .expect("Failure to even connect is no good");
                let _ = resp.text();
                resp
            });
            f(Fact::record(resp, duration));
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
}
