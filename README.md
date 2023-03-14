# renode-run

A custom Cargo runner that runs Rust firmware in the [renode](https://renode.io/) emulator.

## TODOs/ideas

* provide a default env for envsub stuff and the renode cmd
  https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
* uses all cargo env vars where possible/sensible
* add/manage renode's PATH env
* maybe manages downloading renode portable linux package
* runner bin has opts with env var overrides
* use the uart hooks, etc, in an example, panic & test runner triggers
  ```
  logFile @logfile.log true
  logLevel 3 file

  sysbus.usart3 AddLineHook "" "Antmicro.Renode.Logging.Logger.Log(LogLevel.Error, line)"
  sysbus.usart3 AddLineHook "Free heap" "Antmicro.Renode.Emulator.Exit()"

  start
  ```
* toml table supports `variants`, allows multiple configs
  - one for test, has uart hooks for test runner output, panic, etc
  - another for normal app stuff

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.
