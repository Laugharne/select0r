# Select0r


<!-- TOC -->

- [Select0r](#select0r)
	- [Install Rust](#install-rust)
	- [Build](#build)
	- [Run](#run)

<!-- /TOC -->

Find better **EVM function name** to optimize Gas cost !

**TODO**

## Install Rust
[https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)


## Build

Build artifacts in release mode, with optimizations.

`cargo build --release`


## Run

In release directory

`select0r "functionName(uint256,address,...)" (1|2|3) (true|false)`

Usage : `<function_signature string> <difficulty number> <leading_zero boolean>`

Example : `select0r "functionName(uint)" 2 true`
