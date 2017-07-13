ezo-common-rs
=============

A library with shared functionality for the `EZO` chip, made by Atlas Scientific.

>   Currently, only I2C communication is available.

## Requirements
This version needs _nightly_ to compile, since it makes use of `#![feature(inclusive_range_syntax)]`.

This crate makes use of `error-chain` to handle errors. Include it in your dependencies to make use of it.

This crate makes use of `i2cdev` to handle errors. Include it in your dependencies to make use of it.

## Usage

First, add this to your `Cargo.toml`:

```
ezo_common = "0.1.0"
```

## Crates for specific EZO chips

*   [ezo-rtd-rs](https://github.com/saibatizoku/ezo-rtd-rs) RTD EZO Chip - For sensing temperature.
*   [ezo-ec-rs](https://github.com/saibatizoku/ezo-ec-rs) EC EZO Chip - For sensing electric conductivity.
*   [ezo-ph-rs](https://github.com/saibatizoku/ezo-ph-rs) pH EZO Chip - For sensing pH (acidity or alkalinity).
