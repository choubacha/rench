#[derive(Clone, Copy)]
pub struct Plan {
    threads: usize,
    requests: usize,
}

impl Plan {
    pub fn new(threads: usize, requests: usize) -> Self {
        Self { threads, requests }
    }

    pub fn threads(&self) -> usize {
        self.threads
    }

    pub fn requests(&self) -> usize {
        self.requests
    }

    pub fn distribute(&self) -> Vec<usize> {
        // Every thread should get even work:
        let base_work = self.requests / self.threads;
        let remaining_work = self.requests % self.threads;

        (0..self.threads)
            .map(|thread| {
                // The remainder means that we don't have enough for
                // every thread to get 1. So we just add one until
                // we've used up the entire remainder
                if thread < remaining_work {
                    base_work + 1
                } else {
                    base_work
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_can_distribute_all_work_as_evenly_as_possible() {
        assert_eq!(Plan::new(3, 1000).distribute(), vec![334, 333, 333]);
        assert_eq!(Plan::new(2, 1000).distribute(), vec![500, 500]);
        assert_eq!(
            Plan::new(20, 39).distribute(),
            vec![2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 1]
        );
    }
}
