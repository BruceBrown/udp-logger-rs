# udp-logger-rs
[![crates.io version][1]][2] [![build status][3]][4]
[![downloads][5]][6] [![docs.rs docs][7]][8]

Log macro for log's kv-unstable backend.

- [Documentation][8]
- [Crates.io][2]
- [Releases][releases]

## Motivation
I wanted to build key/value context into my structs and log it when logging within an impl fn.
Beyond that, I also wanted to separate the responsiblity of logging from log file management and any
associated business logic.

Consider an architecture with multiple containers running on the same host. One being a
server, the other being a log processor. Through this separation, the server performs logging and
is insulated from the processing of that logging. Meanwhile, the log processor determines
how log messages are processed, which can itself become quite complex.

Additionally, I decided that one might want to map the log level to a source port, and/or destination port.
The thought here is that it might simplify processing, as the source and/or destination port
implies the log level.

Finally, by separating them, server development can use a different log processor than production and
production log processing can be tested independent of the server.

## Examples
```rust
use udp_logger_rs::info;

fn main() {
    udp_logger_rs::UdpLogger::default().init().unwrap();
    info!("hello");
    info!("hello",);
    info!("hello {}", "cats");
    info!("hello {}", "cats",);
    let ctx: Vec<(String, String)> = vec![
        ("cat_1".into(), "chashu".into()),
        ("cat_2".into(), "nori".into()),
    ];
    info!(kvs: &ctx, "hello {}", "cats",);
}
```

The UDP client can be as trivial as:
```rust
use smol::Async;
use std::io::{self};
use std::net::UdpSocket;

fn main() -> io::Result<()> {
    let _result = futures_lite::future::block_on(async {
        let socket = Async::<UdpSocket>::bind(([127, 0, 0, 1], 4010))?;
        let mut buf = [0u8; 1024 * 32];
        loop {
            let (len, addr) = socket.recv_from(&mut buf).await?;
            let logmsg = std::str::from_utf8(&buf[..len]).expect("invalid utf8");
            println!("{} from={}", logmsg, addr);
        }
        // This hack of unreachable code cements the return type
        #[allow(unreachable_code)]
        io::Result::Ok(())
    });
    Ok(())
}
```

## Installation
```sh
$ cargo add udp-logger-rs
```

## Safety
This crate uses ``#![forbid(unsafe_code)]`` to ensure everything is implemented in
100% Safe Rust.

## Contributing
Want to join us? Check out our ["Contributing" guide][contributing] and take a
look at some of these issues:

- [Issues labeled "good first issue"][good-first-issue]
- [Issues labeled "help wanted"][help-wanted]

## References
None.

## License
[MIT](./LICENSE-MIT) OR [Apache-2.0](./LICENSE-APACHE)

[1]: https://img.shields.io/crates/v/udp-logger-rs.svg?style=flat-square
[2]: https://crates.io/crates/udp-logger-rs
[3]: https://api.travis-ci.com/BruceBrown/udp-logger-rs.svg?branch=main
[4]: https://app.travis-ci.com/github/brucebrown/udp-logger-rs
[5]: https://img.shields.io/crates/d/udp-logger-rs.svg?style=flat-square
[6]: https://crates.io/crates/udp-logger-rs
[7]: https://img.shields.io/badge/docs-latest-blue.svg?style=flat-square
[8]: https://docs.rs/udp-logger-rs

[releases]: https://github.com/brucebrown/udp-logger-rs/releases
[contributing]: https://github.com/brucebrown/udp-logger-rs/blob/master.github/CONTRIBUTING.md
[good-first-issue]: https://github.com/brucebrown/udp-logger-rs/labels/good%20first%20issue
[help-wanted]: https://github.com/brucebrown/udp-logger-rs/labels/help%20wanted
