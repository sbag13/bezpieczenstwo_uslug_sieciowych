# Laboratory classes 1 - server-client chat

This is project of implementation chat in client-server architecture with secure secret number exchange using Diffie-Hellman protocol.

Project consists of 3 main modules:

- server
- client
- common library

## Prerequisites

- Cargo - the Rust package manager.
- Rust tools

## Building

To compile any of the mentioned modules enter the module root directory and use cargo to build it.

Example:

``` bash
cd ./server
cargo build
```

## Running

To run any module one can use cargo again:

```bash
cargo run
```

While passing arguments to program using cargo, separate arguments from run command with **--**. For example:

```bash
cargo run -- -nickname John
```

After building programs can be run directly from binary which can be found in **target** folder.

### Options

Client module can be run with options.

```
$ cargo run -- --help
Client-server tcp chat 1.0
Szymon Baginski <baginski.szymon@gmail.com>

USAGE:
    client [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -e, --encryption <NONE|XOR|CEZAR>    Sets encryption method.
    -n, --nickname <NICKNAME>            Sets your nickname.

```



## Unit test

Server module and common library have some UTs. To run them enter the following command from root directory of a module:

``` bash
cargo test -- --no-capture
```

**--no-capture** option corresponds to writing output from tests to stdout.

 ## Author

Szymon Bagiński, baginski.szymon@gmail.com

