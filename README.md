# renode-run

A custom Cargo runner that runs Rust firmware in the [renode] emulator.

## Features

* Acts as a Cargo runner, integrating into `cargo run`.
* Exposes all of [renode]'s scripting facilities and CLI as configuration in your `Cargo.toml`.
* Provides configuration for the environment, allowing you to perform environment substitution on
  nearly everything.

## TODOs/ideas

* uses/sets cargo env vars where possible/sensible: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
* add/manage renode's PATH env
* maybe manages downloading renode portable linux package
* runner bin has opts with env var overrides for config things
* toml table supports `variants`, allows multiple configs
  - one for test, has uart hooks for test runner output, panic, etc
  - another for normal app stuff

## Installation

To install `renode-run`, use `cargo install renode-run`.

## TODO Setup

## TODO Configuration

## TODO Examples

```toml
[package.metadata.renode]
name = 'my-script'
description = 'my renode script - ${FOOKEY} works'
machine-name = 'my-machine'
using-sysbus = true
renode = '${HOME}/repos/forks/renode/renode'
environment-variables = [
    ['FOOKEY', 'FOOVAL'],
    ["MYENV", "MYVAL"],
]
init-commands = [
    'logLevel -1 i2c2',
]
variables = [
    '$tap?="renode-tap0"',
    # Set random board UNIQUE ID
    '''
    python "import _random"
    python "rand = _random.Random()"

    $id1 = `python "print rand.getrandbits(32)"`
    $id2 = `python "print rand.getrandbits(32)"`
    $id3 = `python "print rand.getrandbits(32)"`
    ''',
]
platform-descriptions = [
    '@platforms/boards/stm32f4_discovery-kit.repl',
    'path/to/dev_board.repl',
    '< ${SOMETHING}/other_dev_board.repl',
    '''
    phy3: Network.EthernetPhysicalLayer @ ethernet 3
        Id1: 0x0000
        Id2: 0x0000
    ''',
    '''
    wss: Python.PythonPeripheral @ sysbus 0x50070000
        size: 0x10
        initable: true
        filename: "${ORIGIN}/sensor_models/wss.py"
    ''',
]
pre-start-commands = [
    '''
    emulation CreateSwitch "switch"
    connector Connect sysbus.ethernet switch
    emulation CreateTap $tap "tap"
    connector Connect host.tap switch
    ''',
    '''
    logFile @/tmp/logfile.log true
    logLevel 3 file
    ''',
    'emulation LogEthernetTraffic',
    'machine StartGdbServer 3333',
]
reset = '''
sysbus LoadELF $bin
sysbus WriteDoubleWord 0x1FFF7A10 $id1
sysbus WriteDoubleWord 0x1FFF7A14 $id2
sysbus WriteDoubleWord 0x1FFF7A18 $id3
'''
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)

- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall be
licensed as above, without any additional terms or conditions.

[renode]: https://renode.io/
