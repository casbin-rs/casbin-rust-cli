# casbin-rust-cli

[![Crates.io](https://img.shields.io/crates/v/casbin-rust-cli.svg)](https://crates.io/crates/casbin-rust-cli)
[![Docs](https://docs.rs/casbin-rust-cli/badge.svg)](https://docs.rs/casbin-rust-cli)
[![CI](https://github.com/casbin-rs/casbin-rust-cli/actions/workflows/ci.yml/badge.svg)](https://github.com/casbin-rs/casbin-rust-cli/actions/workflows/ci.yml)

casbin-rust-cli is a command-line tool based on Casbin (Rust language), enabling you to use all of Casbin APIs in the shell.

## Installation

### From crates.io
```shell
cargo install --locked casbin-rust-cli
```

### Install Manually
```shell
git clone https://github.com/casbin-rs/casbin-rust-cli.git
cd casbin-rust-cli
cargo install --path .
```

## Options
| options        | description                                  | must |                    
|----------------|----------------------------------------------|------|
| `-m, --model`  | The path of the model file or model text     | y    |
| `-p, --policy` | The path of the policy file or policy text   | y    |  
| `enforce`      | Check permissions                            | n    |

## Get started

- Check whether Alice has read permission on data1

    ```shell
    ./casbin-rust-cli enforce -m "examples/basic_model.conf" -p "examples/basic_policy.csv" "alice" "data1" "read"
    ```
    > {"allow":true,"explain":[]}