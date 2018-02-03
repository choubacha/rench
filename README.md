# RENCH

A benchmark utility for end points. Written in rust... rust + bench = rench

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
