# renode-run &emsp; ![ci] [![crates.io]](https://crates.io/crates/renode-run)

A custom Cargo runner that runs Rust firmware in the [renode] emulator.

## Features

* Acts as a Cargo runner, integrating into `cargo run`.
* Exposes all of [renode]'s scripting facilities and CLI as configuration in your `Cargo.toml`.
* Provides configuration for the environment, allowing you to perform environment variable
  substitution on nearly everything.

## Installation

To install `renode-run`, use `cargo install renode-run`.

## Setup

**NOTE** Requires [renode] to be installed on the host system.

### 1. Set the Cargo runner

Set `renode-run` as your Cargo runner (`.cargo/config.toml`).

``` toml
[target.'cfg(all(target_arch = "arm", target_os = "none"))']
runner = "renode-run"
```

### 2. Run

You can now run your firmware using `cargo run`.

## Configuration

### `[package.metadata.renode]` options

**NOTE** Many of these can be overridden by CLI options. Nearly every field supports environment variable substitution.

Fields pertaining the `resc` script generation:
- **name**: The name field used in the generated `resc` script.
  If not present, the name of the crate is used or a default name is provided.
- **description**: The description field used in the generated `resc` script.
  If not present, the description of the crate is used or a default description is provided.
- **machine-name**: The machine's name.
- **init-commands**: An array of commands ran after the machine is created and before variables are declared.
- **variables**: An array of variable declarations.
  `renode-run` will automatically insert `$bin = @target/<profile>/<bin>` as provided by Cargo.
- **platform-description**: A single platform description.
  Can be one of:
    * a renode-provided `repl` file (starts with `@`)
    * a local `repl` file (doesn't start with `@`)
    * a local `repl` file that is to be imported and generated into the output directory (starts with `<`).
      This is handy when you want to perform environment substitution on the contents of a `repl` file.
    * a literal string
- **platform-descriptions**: An array of platform descriptions.
  Each entry can be one of:
    * a renode-provided `repl` file (starts with `@`)
    * a local `repl` file (doesn't start with `@`)
    * a local `repl` file that is to be imported and generated into the output directory (starts with `<`).
      This is handy when you want to perform environment substitution on the contents of a `repl` file.
    * a literal string
- **reset**: The reset macro definition. The default is `sysbus LoadELF $bin`.
- **pre-start-commands**: An array of commands ran immediately before the `start` command.
- **post-start-commands**: An array of commands ran immediately after the `start` command.

Fields pertaining the behavior of `renode-run`:
- **environment-variables**: An array of environment variables and values to set for both the `renode-run` and `renode` environment.
- **renode**: The path to the `renode` binary to use. Defaults to using the system's `$PATH`.
- **omit-start**: Don't generate a `start` command. Defaults to `false`.
- **omit-out-dir-path**: Don't add the output directory to renode's path.
- **resc-file-name**: The name of the generated `resc` script. Defaults to `emulate.resc`.
- **use-relative-paths**: TBD
- **disable-envsub**: TBD
- **using-sysbus**: TBD

Fields pertaining the invocation of `renode` itself:
- **plain**: Adds `--plain` to the list of arguments given to `renode`.
- **port**: Adds `--port <port>` to the list of arguments given to `renode`.
- **disable-xwt**: Adds `--disable-xwt` to the list of arguments given to `renode`.
- **hide-monitor**: Adds `--hide-monitor` to the list of arguments given to `renode`.
- **hide-log**: Adds `--hide-log` to the list of arguments given to `renode`.
- **hide-analyzers**: Adds `--hide-analyzers` to the list of arguments given to `renode`.
- **console**: Adds `--console` to the list of arguments given to `renode`.
- **keep-temporary-files**: Adds `--keep-temporary-files` to the list of arguments given to `renode`.

## Example

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
    '''
    sysbus.usart3 AddLineHook "PANIC" "Antmicro.Renode.Logging.Logger.Log(LogLevel.Error, line)"
    sysbus.usart3 AddLineHook "test result: ok" "Antmicro.Renode.Emulator.Exit()"
    ''',
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

[ci]: https://github.com/jonlamb-gh/renode-run/workflows/CI/badge.svg
[crates.io]: https://img.shields.io/crates/v/renode-run.svg
[renode]: https://renode.io/
