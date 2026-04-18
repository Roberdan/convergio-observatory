# Changelog

## [0.1.8](https://github.com/Roberdan/convergio-observatory/compare/v0.1.7...v0.1.8) (2026-04-18)


### Bug Fixes

* **deps:** bump rustls-webpki 0.103.11 -&gt; 0.103.12 (RUSTSEC-2026-0099) ([552d90a](https://github.com/Roberdan/convergio-observatory/commit/552d90a1b7ebdb2fea87dcc93b5c38c881f2a332))
* security and quality audit pass 2 ([3841ebb](https://github.com/Roberdan/convergio-observatory/commit/3841ebb5b0eecadded3a816e96735f18941854d2))
* SSRF bypass via userinfo, info leak, error swallowing ([7bb7302](https://github.com/Roberdan/convergio-observatory/commit/7bb73024675da280c6e6391d680bf4c2e974a06b))

## [0.1.7](https://github.com/Roberdan/convergio-observatory/compare/v0.1.6...v0.1.7) (2026-04-13)


### Bug Fixes

* pass CARGO_REGISTRY_TOKEN to release workflow ([14649e6](https://github.com/Roberdan/convergio-observatory/commit/14649e6d6d2336c25c3e97401608e632bfde5415))

## [0.1.6](https://github.com/Roberdan/convergio-observatory/compare/v0.1.5...v0.1.6) (2026-04-13)


### Bug Fixes

* add crates.io publishing metadata (description, repository) ([fa344eb](https://github.com/Roberdan/convergio-observatory/commit/fa344eb769e5b9d5afe323677fc6cafa30c42428))

## [0.1.5](https://github.com/Roberdan/convergio-observatory/compare/v0.1.4...v0.1.5) (2026-04-13)


### Bug Fixes

* fix malformed convergio-ipc dependency in Cargo.toml ([#8](https://github.com/Roberdan/convergio-observatory/issues/8)) ([10999c0](https://github.com/Roberdan/convergio-observatory/commit/10999c0d4c2a7ff7b3c498a64d9a72418dccf5bc))

## [0.1.4](https://github.com/Roberdan/convergio-observatory/compare/v0.1.3...v0.1.4) (2026-04-13)


### Bug Fixes

* **deps:** update convergio-ipc to v0.1.6 (SDK v0.1.9 aligned) ([3f6ec8b](https://github.com/Roberdan/convergio-observatory/commit/3f6ec8b7848520e1f34521934f75a2cc41c2e102))

## [0.1.3](https://github.com/Roberdan/convergio-observatory/compare/v0.1.2...v0.1.3) (2026-04-13)


### Features

* adapt convergio-observatory for standalone repo ([affa8c2](https://github.com/Roberdan/convergio-observatory/commit/affa8c2eeaea898a44e63cbcd4257fa13bac49ac))


### Bug Fixes

* **release:** use vX.Y.Z tag format (remove component) ([88581f7](https://github.com/Roberdan/convergio-observatory/commit/88581f72b59d73a58e07235acfa612980c1b968b))
* security audit — SSRF, info leak, prometheus injection, FTS5 injection ([#2](https://github.com/Roberdan/convergio-observatory/issues/2)) ([9f6e3aa](https://github.com/Roberdan/convergio-observatory/commit/9f6e3aa7e63a406d84fbeb1f4674a309909e8b8b))


### Documentation

* copy ADR from monorepo ([3670127](https://github.com/Roberdan/convergio-observatory/commit/3670127c43ddede835a38c37ac256bfa47cb2fb4))

## [0.1.2](https://github.com/Roberdan/convergio-observatory/compare/convergio-observatory-v0.1.1...convergio-observatory-v0.1.2) (2026-04-12)


### Documentation

* copy ADR from monorepo ([3670127](https://github.com/Roberdan/convergio-observatory/commit/3670127c43ddede835a38c37ac256bfa47cb2fb4))

## [0.1.1](https://github.com/Roberdan/convergio-observatory/compare/convergio-observatory-v0.1.0...convergio-observatory-v0.1.1) (2026-04-12)


### Features

* adapt convergio-observatory for standalone repo ([affa8c2](https://github.com/Roberdan/convergio-observatory/commit/affa8c2eeaea898a44e63cbcd4257fa13bac49ac))


### Bug Fixes

* security audit — SSRF, info leak, prometheus injection, FTS5 injection ([#2](https://github.com/Roberdan/convergio-observatory/issues/2)) ([9f6e3aa](https://github.com/Roberdan/convergio-observatory/commit/9f6e3aa7e63a406d84fbeb1f4674a309909e8b8b))

## 0.1.0 (Initial Release)

### Features

- Initial extraction from convergio monorepo
