# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](http://keepachangelog.com/en/1.0.0/)
and this project adheres to [Semantic Versioning](http://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

* Chart size option was added allowing specification of large, medium, small, or none.
* Added the calculations and renderings of the standard deviation of the request latency.

## [0.2.0] - 2018-04-08

### Added

* Counts up the returned status codes and presents them to the user.
* Can specify `-i` or `--head` to use the http method HEAD instead of GET.
* Add ability to use different engines. Specifically, the `hyper` crate can now be used to issue requests as opposed to `reqwest` which wraps it.

### Fixed

* Uses the byte length of the body to calculate data transferred instead of the Content-Length header which can be absent for certain encodings.
