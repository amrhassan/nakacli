# nakacli #
[![Build Status](https://travis-ci.org/amrhassan/nakacli.svg?branch=master)](https://travis-ci.org/amrhassan/nakacli)
[![Released Version](https://img.shields.io/crates/v/nakacli.svg)](https://crates.io/crates/nakacli)

> **CLI Client for [Nakadi](https://zalando.github.io/nakadi/)** - Cross-platform, no-depdency minimal CLI for interacting with the Nakadi message broker

## Install ##
### macOS ###
```bash
brew install amrhassan/macosapps/nakacli
```
### Ubuntu ###
```bash
snap install --edge nakacli
```
### Arch Linux ###
```bash
yaourt -S nakacli-bin   # Or substitute with your favorite AUR helper
```

### Other Platforms ###
`nakacli` compiles into a single executable binary with no extra runtime dependencies, so find the latest release in [releases](https://github.com/amrhassan/nakacli/releases) and run it however you run binaries on your operating system.

## Features ##
- [x] Metrics querying
- [x] [Zign](https://github.com/zalando-stups/zign) authentication
- [ ] Event type creation
- [ ] Even type deletion
- [x] Publishing events
- [x] Stream published events of a certain type
- [ ] Creating subscriptions
- [ ] Stream-listening on events from a subscription

## Usage ##
### Commands ###
#### `nakacli event publish [FLAGS] [OPTIONS] <event-type> <json-body>` ####
Publishes one or more events of the type `<event-type>`. The `<json-body>` can be the full body of a single event as a JSON object, or a JSON array containing an object for each event to be published.

#### `nakacli event stream [FLAGS] [OPTIONS] <event-type>` ####
Starts streaming published events of type `<event-type>` to STDOUT. Should never stop unless interrupted by the user.

#### `nakacli metrics [FLAGS] [OPTIONS]` ####
Prints the Nakadi server metrics.

### Global options/flags ###
#### `--bearer-token <TOKEN>` and `--zign` ####
For any command, you can specify a Bearer token via the `--bearer-token <TOKEN>` option or the `BEARER_TOKEN` environment variable.

If you have [Zign](https://github.com/zalando-stups/zign) set up, you can use it by simply passing the `--zign` flag.

#### `--url <NAKADI_URL>` ####
Specifies the URL to the Nakadi server in the format `scheme://[auth:]hostname:[port]`. It can also be set via the `NAKADI_URL` environment variable.

#### `--pretty` ####
Makes JSON output properly-indented for easier human readability.

### More ###
Check `nakacli help` for a full list of all the supported commands, their options, flags and arguments.
