[![crates.io](https://img.shields.io/crates/v/smartoris-gestures.svg)](https://crates.io/crates/smartoris-gestures)
![maintenance](https://img.shields.io/badge/maintenance-actively--developed-brightgreen.svg)

# smartoris-gestures

Hand gestures sensing library for [Drone OS].

[Drone OS]: https://www.drone-os.com/

## Usage

Add the crate to your `Cargo.toml` dependencies:

```toml
[dependencies]
smartoris-gestures = { version = "0.1.0" }
```

Add or extend `std` feature as follows:

```toml
[features]
std = ["smartoris-gestures/std"]
```

## License

Licensed under either of

 * Apache License, Version 2.0
   ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license
   ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
dual licensed as above, without any additional terms or conditions.
