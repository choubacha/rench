#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Plan {
    threads: usize,
    requests: usize,
    urls: Vec<String>,
    method: Method,
}

/// The methods that are supported by the current implementations. These are currently
/// body-less methods so that we don't need to load up any additional content.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum Method {
    Get,
    Head,
}

/// A plan builder.
pub struct Builder {
    plan: Plan,
}

impl Builder {
    /// Creates a new builder.
    pub fn new() -> Self {
        Self {
            plan: Plan::default(),
        }
    }

    /// Adds a url to the plan.
    pub fn with_url(&mut self, url: &str) -> &mut Self {
        self.plan.urls.push(url.to_string());
        self
    }

    /// Change the method the built plan should have.
    pub fn with_method(&mut self, method: Method) -> &mut Self {
        self.plan.method = method;
        self
    }

    /// Change the number of requests that the plan will issue.
    pub fn with_requests(&mut self, requests: usize) -> &mut Self {
        self.plan.requests = requests;
        self
    }

    /// Change how many threads the plan will use.
    pub fn with_threads(&mut self, threads: usize) -> &mut Self {
        self.plan.threads = threads;
        self
    }

    /// Builds and returns a new Plan
    pub fn build(&self) -> Plan {
        self.plan.clone()
    }
}

impl Plan {
    fn default() -> Plan {
        Plan {
            threads: 1,
            requests: 1,
            method: Method::Get,
            urls: Vec::new(),
        }
    }

    /// Creates a new builder for a plan.
    pub fn builder() -> Builder {
        Builder::new()
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    pub fn requests(&self) -> usize {
        self.requests
    }

    pub fn urls(&self) -> &[String] {
        &self.urls
    }

    pub fn method(&self) -> Method {
        self.method
    }

    pub fn distribute(&self) -> Vec<Plan> {
        if self.requests < 1 || self.threads < 1 {
            return Vec::new();
        }

        // Every thread should get even work:
        let base_work = self.requests / self.threads;
        let remaining_work = self.requests % self.threads;

        (0..self.threads)
            .map(|thread| {
                // The remainder means that we don't have enough for
                // every thread to get 1. So we just add one until
                // we've used up the entire remainder
                let requests = if thread < remaining_work {
                    base_work + 1
                } else {
                    base_work
                };
                Plan {
                    requests,
                    ..self.clone()
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_builds_a_plan() {
        let mut builder = Plan::builder();
        assert_eq!(builder.build().threads(), 1);
        assert_eq!(builder.build().requests(), 1);
        assert_eq!(builder.build().method(), Method::Get);
        assert!(builder.build().urls().is_empty());

        builder.with_url("http://www.google.com");
        assert_eq!(
            builder.build().urls(),
            &[String::from("http://www.google.com")]
        );

        builder.with_method(Method::Head);
        assert_eq!(builder.build().method(), Method::Head);

        builder.with_requests(10);
        assert_eq!(builder.build().requests(), 10);

        builder.with_threads(11);
        assert_eq!(builder.build().threads(), 11);

        builder.with_requests(123).with_threads(456);
        assert_eq!(builder.build().requests(), 123);
        assert_eq!(builder.build().threads(), 456);
    }

    #[test]
    fn it_can_distribute_all_work_as_evenly_as_possible() {
        fn map_it(plan: Plan) -> Vec<usize> {
            plan.distribute().iter().map(|s| s.requests).collect()
        }

        let mut builder = Plan::builder();
        builder.with_requests(1000);
        assert_eq!(map_it(builder.with_threads(3).build()), vec![334, 333, 333]);
        assert_eq!(map_it(builder.with_threads(2).build()), vec![500, 500]);
        assert_eq!(
            map_it(builder.with_requests(39).with_threads(20).build()),
            vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1]
        );
    }
}
