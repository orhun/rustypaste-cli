<a href="https://github.com/orhun/rustypaste-cli"><img src="img/logo.png" width="500"></a>

A CLI tool for [**rustypaste**](https://github.com/orhun/rustypaste).

![demo](img/demo.gif)

## Installation

### From crates.io

```sh
cargo install rustypaste-cli
```

### Binary releases

See the available binaries on [releases](https://github.com/orhun/rustypaste-cli/releases/) page.

### Build from source

```sh
git clone https://github.com/orhun/rustypaste-cli.git
cd rustypaste-cli/
cargo build --release
```

## Usage

`rpaste [options] <file(s)>`

```
-h, --help          prints help information
-v, --version       prints version information
-o, --oneshot       generates one shot links
-p, --pretty        prettifies the output
-c, --config CONFIG sets the configuration file
-s, --server SERVER sets the address of the rustypaste server
-a, --auth TOKEN    sets the authentication token
-u, --url URL       sets the URL to shorten
-e, --expire TIME   sets the expiration time for the link
```

### Set credentials

Either set the credentials on the command line (not recommended):

```sh
rpaste -s "https://paste.example.com" -a "<token>"
```

or specify them in the [configuration file](#configuration).

### Upload files

```sh
rpaste awesome.txt other.txt
```

### Shorten URLs

```sh
rpaste -u https://example.com/some/long/url
```

### One shot

```sh
rpaste -o disappear_after_seen.txt
```

### Expiration

```sh
rpaste -e 10min expires_in_10_minutes.txt
```

```sh
rpaste -e 1hour -u https://example.com/expire/1hour
```

\* Supported units: `ns`, `us`, `ms`, `sec`, `min`, `hours`, `days`, `weeks`, `months`, `years`

### Extras

* Show a _prettier_ output: `rpaste -p [...]`
* [Disable colors](https://no-color.org/) in the output: `NO_COLOR=1 rpaste -p [...]`

## Configuration

The configuration file can be specified via `--config` argument and `RPASTE_CONFIG` environment variable or it can be placed to the following global location:

* `$HOME/.rustypaste/config.toml`

See [config.toml](./config.toml) for configuration options.

## Contributing

Pull requests are welcome!

#### License

<sup>
All code is licensed under <a href="LICENSE">The MIT License</a>.
</sup>
