---
hide_side_table_of_contents: true
---
# Installation

## Docker (all platforms)

A pre-built Docker image is available for all systems from Docker Hub.

```shell
$ docker run --rm scalesocket/scalesocket:latest scalesocket --help
```

## Cargo (all platforms)

If you have installed the Rust toolchain, you can also install using cargo (requires nightly):

```shell
cargo install --force scalesocket
```

## Binary release (linux)

Binary releases are available for linux via [GitHub](https://github.com/scalesocket/scalesocket/releases), or using the command line:

```shell
$ VERSION=v0.1.4 curl -SL "https://github.com/scalesocket/scalesocket/releases/download/${VERSION}/scalesocket_${VERSION}_x86_64-unknown-linux-musl.tar.gz" | tar -xzC /usr/bin
```


