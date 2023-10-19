# Select0r

<!-- TOC -->

- [Select0r](#select0r)
	- [Install Rust](#install-rust)
	- [Build](#build)
	- [Run](#run)
	- [Usage, parameters and results](#usage-parameters-and-results)
		- [Usage](#usage)
		- [Parameters](#parameters)
		- [Results](#results)

<!-- /TOC -->

**Select0r - Selector value optimizer**, find better EVM function name to optimize Gas cost.


## Install Rust

[https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)


## Build

Build artifacts in release mode, with optimizations.

`cargo build --release`


## Run

Select `release` directory as working directory.

`select0r "functionName(uint256,address,...)" (1|2|3) (true|false)`


## Usage, parameters and results


### Usage

select0r s <function_signature string> z <number_of_zeros> r <max_results> d <decrement boolean> t <nbr_threads> o <format_ouput>


### Parameters

| Parameters | Full names           | Data types | Examples      | Domains          | Default       | Descriptions                 |
| ---------- | -------------------- | ---------- | ------------- | ---------------- | ------------- | ---------------------------- |
| **`s`**    | `function_signature` | string     | mint(address) | *(1)*            | **Mandatory** | Function signature *(1)*     |
| **`z`**    | `number_of_zeros`    | numeric    | 2             | [1..3]           | **2**         | # of zero (difficulty) *(2)* |
| **`r`**    | `max_results`        | numeric    | 5             | [2..10]          | **4**         | # of needed result *(2)*     |
| **`d`**    | `decrement`          | boolean    | true          | true/false       | **false**     | *(3)*                        |
| **`t`**    | `nbr_threads`        | numeric    | 4             | [2..#cpu]        | **2**         | # of threads to use (*4*)    |
| **`o`**    | `format_ouput`       | string     | xml           | tsv/csv/json/xml | **tsv**       | File format output           |

- *(1) : a*
- *(2) : higher it is, longer it is*
- *(3) : slower if true*
- *(4) : hardware limitation (#CPU)*

- **Example 1** : `select0r s "functionName(uint256)"  z 2  r 5  d true  t 2  o tsv`
- **Example 2** : `select0r s "functionName2(uint)"  z 2  r 7  d false  t 2  o json`


### Results

Get results for `mint(address)` with `2` lead zeros minimum, `sorted` by selector value, using `3` threads, stop after`4` results and put it as an `XML` file.

```bash
select0r s "mint(address)"  z 2  d true  t 3  r 4  o xml
```

**File :  ** `mint(address)--zero=2-max=4-decr=true-cpu=3.XML`

```xml
<?xml version="1.0" encoding="UTF-8"?>
<select0r>
		<result>
				<selector>00a08a00</selector>
				<leading_zero>1</leading_zero>
				<signature>mint_Yh(address)</signature>
		</result>
		<result>
				<selector>00009e37</selector>
				<leading_zero>2</leading_zero>
				<signature>mint_6X1(address)</signature>
		</result>
		<result>
				<selector>000032d8</selector>
				<leading_zero>2</leading_zero>
				<signature>mint_AeL(address)</signature>
		</result>
		<result>
				<selector>0000129b</selector>
				<leading_zero>2</leading_zero>
				<signature>mint_TWX(address)</signature>
		</result>
</select0r>
```
