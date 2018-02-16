# RENCH

A benchmark utility for end points. Written in rust... rust + bench = rench

[![CircleCI](https://img.shields.io/circleci/project/github/kbacha/rench.svg)](https://circleci.com/gh/kbacha/rench)
[![Travis](https://img.shields.io/travis/kbacha/rench.svg)](https://travis-ci.org/kbacha/rench)
[![Crates.io](https://img.shields.io/crates/v/rench.svg)](https://crates.io/crates/rench)
[![Crates.io](https://img.shields.io/crates/d/rench.svg)](https://crates.io/crates/rench)
[![Crates.io](https://img.shields.io/crates/l/rench.svg)](https://crates.io/crates/rench)

# Installation

```
cargo install -f
```

# Usage

The gist of a http benchmarker is to run a series of queries against an endpoint
and collect the facts from it. These facts then are summarized for the user to
examine. To really maximize an end point, we use a simple threading model to make
many requests at the same time. This allows us to generate a large number of
requests and try to fully saturate the http end point.

You can change the number of threads and the number of requests to suit your needs.
You can even specify multiple URLs and it will round-robin the requests between them.

```bash
$ ./target/release/rench -c 1 --engine=hyper -n 10000 http://0.0.0.0:6767
Beginning requests
1000 requests
2000 requests
3000 requests
4000 requests
5000 requests
6000 requests
7000 requests
8000 requests
9000 requests
10000 requests
Finished!

Took 1.770175211 seconds
5649.158308092475 requests / second

Summary
  Average:   0.170058 ms
  Median:    0.131708 ms
  Longest:   4.17715 ms
  Shortest:  0.075699 ms
  Requests:  10000
  Data:      4.73 MB

Latency Percentiles (2% of requests per bar):
                                                 ▌ 0.203628
                                              ▖▖▌▌
                                  ▖▖▖▖▖▖▖▌▌▌▌▌▌▌▌▌
                  ▖▖▖▖▖▖▖▖▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌
      ▖▖▖▖▖▖▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌
 ▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌
▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌
▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌
▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌
▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌▌ 0


Latency Histogram (each bar is 2% of max latency)
 ▌                                                 9328
 ▌
 ▌
 ▌
 ▌
 ▌
 ▌
 ▌
 ▌
▖▌▌▖ ▖  ▖   ▖▖   ▖  ▖▖▖▖▖▖▖▖▖▖▖▖▖▖▖▖   ▖  ▖▖▖▖ ▖▖▖ 0
```

### Options

```bash
$ rench --help
Git Release Names
Kevin Choubacha <chewbacha@gmail.com>

USAGE:
    rench [OPTIONS] <URL>...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -c <concurrency>
    -n <requests>

ARGS:
    <URL>...
```
