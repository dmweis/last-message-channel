# Last Message Channel

[![codecov](https://codecov.io/gh/dmweis/last-message-channel/branch/main/graph/badge.svg)](https://codecov.io/gh/dmweis/last-message-channel)
[![Rust](https://github.com/dmweis/last-message-channel/workflows/Rust/badge.svg)](https://github.com/dmweis/last-message-channel/actions)
[![Security audit](https://github.com/dmweis/last-message-channel/workflows/Security%20audit/badge.svg)](https://github.com/dmweis/last-message-channel/actions)
[![Private docs](https://github.com/dmweis/last-message-channel/workflows/Deploy%20Docs%20to%20GitHub%20Pages/badge.svg)](https://davidweis.dev/last-message-channel/last-message-channel/index.html)

Channel that only holds the latest message

## Disclaimer

I wrote this because I found myself needing a channel like this a lot while writing robot code. It's very simple. While it is covered by tests. It's parallel code so it's very likely that there may be bugs that simple unit tests won't discover

## Dependency

To add as dependency the following line has to be added to Cargo.toml

```toml
last-message-channel = { branch = "main", git = "https://github.com/dmweis/last-message-channel" }
```
