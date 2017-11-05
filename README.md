# nakacli #
[![Build Status](https://travis-ci.org/amrhassan/nakacli.svg?branch=master)](https://travis-ci.org/amrhassan/nakacli)
[![Released Version](https://img.shields.io/crates/v/nakacli.svg)](https://crates.io/crates/nakacli)

CLI Client for [Nakadi](https://zalando.github.io/nakadi/)

# Install #
## macOS ##
```bash
brew install amrhassan/macosapps/nakacli
```
## Ubuntu ##
```bash
snap install --edge nakacli
```
## Arch Linux ##
```bash
yaourt -S nakacli-bin   # Or substitute with your favorite AUR helper
```

## Other Platforms ##
`nakacli` compiles into a single binary with no extra runtime dependencies, so find the latest release in [releases](https://github.com/amrhassan/nakacli/releases) and run it however you run binaries on your operating system.

# Features #
- [x] Metrics querying
- [x] [Zign](https://github.com/zalando-stups/zign) authentication
- [ ] Event type creation
- [ ] Even type deletion
- [x] Publishing events
- [ ] Creating subscriptions
- [ ] Stream-listening on events from a subscription
# Usage Examples #

## Publishing an event ##
To publish an event of the type named `special-event` with the example JSON data:
```json
{"n1": 55, "quantity": 800, "details": "The event has happened"}
```
```bash
nakacli event publish special-event '{"n1": 55, "quantity": 800, "details": "The event has happened"}'
```
The JSON body can be a JSON Object with a single event's data or a JSON Array containing a JSON Object for each event to be published.

## Authorization ##
You could specify a Bearer token via the `--bearer-token` flag or the `BEARER_TOKEN` environment variable.

```bash
nakacli --bearer-token=(secret_token) metrics
```
```bash
export BEARER_TOKEN=(secret_token)
nakacli metrics
```

If you have [Zign](https://github.com/zalando-stups/zign) set up, you can use it by simply passing the `--zign` flag.
```bash
nakacli --zign metrics
```
## More ##
Chec `nakacli --help` for a full list of all the supported commands, their options, flags and arguments.
