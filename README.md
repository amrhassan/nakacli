# nakacli #
[![Build Status](https://travis-ci.org/amrhassan/nakacli.svg?branch=master)](https://travis-ci.org/amrhassan/nakacli)
[![Released Version](https://img.shields.io/crates/v/nakacli.svg)](https://crates.io/crates/nakacli)

> **CLI Client for [Nakadi](https://zalando.github.io/nakadi/)** - Cross-platform, no-dependency minimal CLI for interacting with Nakadi

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
aurman -S nakacli-bin   # Or substitute with your favorite AUR helper
```

### Other Platforms ###
`nakacli` compiles into a single executable binary with no extra runtime dependencies (except for openssl), so find the latest release in [releases](https://github.com/amrhassan/nakacli/releases) and run it however you run binaries on your operating system.

## Features ##
- [x] Metrics querying
- [x] [Zign](https://github.com/zalando-stups/zign) authentication
- [x] Event type creation
- [x] Even type deletion
- [x] Publishing events
- [x] Stream published events of a certain type
- [ ] Creating subscriptions
- [ ] Stream-listening on events from a subscription

## Usage ##
### Commands ###
#### `nakacli event publish [FLAGS] [OPTIONS] <event-type> <json-body>` ####
Publishes one or more events of the type `<event-type>`. The `<json-body>` can be the full body of a single event as a JSON object, or a JSON array containing an object for each event to be published.

You can specify a file path to read the event body from by passing `@FILEPATH` as the `<json-body>` value.

**Data Change events**

To publish a Data Change event, you'll have to provide one of the options: `--data-create`,
`--data-update`, `--data-delete`, or `--data-snapshot`, otherwise your command will not be, accepted by the Nakadi server.

**Business events**
To publish a Business event, you'll have to provide the `--business` option, otherwise your
command will not be accepted by the Nakadi server.

#### `nakacli event stream [FLAGS] [OPTIONS] <event-type>` ####
Starts streaming published events of type `<event-type>` to STDOUT. It should block while it's streaming published events until it is interrupted by the user, or it has consumed `N` number of events where `N` is provide by the `--take=N` option.

#### `nakacli event-type create [FLAGS] [OPTIONS] <owning-application> <name> <json-schema>` ####
Creates an event type with the given parameters. Optionally accepts a `--partition-strategy=hash` param, with which you'll have to specify one or more `--partition-key-field` to indicate the fields to be used in computing the partitioning hash. Compatibility mode for created event type can be specified using the `--compatibility-mode` option.

You can specify a file path to read the JSON Schema from by passing `@FILEPATH` as the `<json-schema>` value.

#### `nakacli event-type delete [FLAGS] [OPTIONS] <name>` ####
Deletes the event-type with the specified `<name>`.

#### `nakacli event-type list [FLAGS] [OPTIONS]` ####
Lists all the registered available event-types (most likely a lot of output, prepare to pipe it to `less`. Also the `--pretty` flag might make it more human-readable if you're interested in reading it.)

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

#### `--network-timeout <network-timeout>` ####
Specifies the network timeout for non-streaming operations in seconds (default value: 1).

### More ###
Check `nakacli help` for a full list of all the supported commands, and `nakacli COMMAND --help` for their options, flags and arguments.
