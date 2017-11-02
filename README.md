# nakacli #
[![Build Status](https://travis-ci.org/amrhassan/nakacli.svg?branch=master)](https://travis-ci.org/amrhassan/nakacli)

CLI Client for Nakadi

# Install #
## macOS ##
```bash
brew install amrhassan/macosapps/nakacli
```
## Arch Linux ##
```bash
yaourt -S nakacli-bin   # Or substitute with your favorite AUR helper
```

## Other Platforms ##
`nakacli` compiles into a single binary with no runtime dependencies, so find the latest release in [releases](https://github.com/amrhassan/nakacli/releases) and run it however you run binaries on your operating system.

# Usage #
```
user$ nakacli --help
CLI Client for Nakadi 

USAGE:
    nakacli [OPTIONS] <SUBCOMMAND>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
        --oauth2-token <TOKEN>     OAuth2 token value
        --url <NAKADI_URL_BASE>    scheme://hostname:[port] of the Nakadi server

SUBCOMMANDS:
    help       Prints this message or the help of the given subcommand(s)
    metrics    Gets monitoring metrics
```
