# Changelog
All notable changes will be kept here.

## [0.2.0] - 2018-04-08

### Added

* Counts up the returned status codes and presents them to the user.
* Can specify `-i` or `--head` to use the http method HEAD instead of GET.
* Add ability to use different engines. Specifically, the `hyper` crate can now be used to issue requests as opposed to `reqwest` which wraps it.

### Fixed

* Uses the byte length of the body to calculate data transferred instead of the Content-Length header which can be absent for certain encodings.
