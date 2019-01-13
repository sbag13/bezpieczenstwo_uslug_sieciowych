# Laboratory classes 1 - server-client chat

This is project of implementation chat in client-server architecture with secure secret number exchange using Diffie-Hellman protocol.

Project consists of 3 main modules:

- server
- client
- common library

## Prerequisites

- Cargo - the Rust package manager.
- Rust tools

Here is how to install them:
https://doc.rust-lang.org/cargo/getting-started/installation.html

## Building

To compile any of the mentioned modules enter the module root directory and use cargo to build it.

Example:

``` bash
cd ./server
cargo build
```

To build release version **--release** flag can be added.

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

Above commands can be preceded with flags which turns on printing logs:
- RUST_LOG=debug
- RUST_LOG=info
- RUST_LOG=error

### Options

Client and server modules can be run with some options.

Client help:

```
Client tcp chat 1.0
Szymon Baginski <baginski.szymon@gmail.com>

USAGE:
    client [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --address <ADDRESS>              Sets ipv4 address of server. Default is 127.0.0.1
    -e, --encryption <NONE|XOR|CEZAR>    Sets encryption method.
    -n, --nickname <NICKNAME>            Sets your nickname. "anonymous" if not given
    -p, --port <PORT>                    Sets port of server. Default is 12345
```

Example:

```
./client -a 192.168.1.2 -p 12356 -n John -e XOR
```



Server help:

```
server tcp chat 1.0
Szymon Baginski <baginski.szymon@gmail.com>

USAGE:
    server [OPTIONS]

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -a, --address <ADDRESS>    Sets ipv4 address of server. Default is 0.0.0.0
    -p, --port <PORT>          Sets port for server. Default is 12345
```

Example:

```
./server -a 0.0.0.0 -p 12345
```



## Unit test

Server module and common library have some UTs. To run them enter the following command from root directory of a module:

``` bash
cargo test -- --no-capture
```

**--no-capture** option corresponds to writing output from tests to stdout.

 ## Author

Szymon Bagiński, baginski.szymon@gmail.com

