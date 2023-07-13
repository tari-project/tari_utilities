# Tari Utilities

![](https://github.com/tari-project/tari_utilities/workflows/Security%20audit/badge.svg)
![](https://github.com/tari-project/tari_utilities/workflows/Clippy/badge.svg)
[![Coverage Status](https://coveralls.io/repos/github/tari-project/tari_utilities/badge.svg)](https://coveralls.io/github/tari-project/tari_utilities)

This crate is part of the [Tari Cryptocurrency](https://tari.com) project.

A set of useful and commonly used utilities that are used in several places in the Tari project.

## bit

Functions for conversion between integer and bit array.

## byte_array

A trait that offers representation of data types as a byte array or hex string. See also extend_bytes and message_format.

## convert

Function which tries to convert a series of `T`s to `U`s.

## encoding

A trait that handles base58 encoding and decoding.

## epoch_time

Data structure representing time as a `u64`.

## extend_bytes

A trait allows us to call append_raw_bytes and get the raw bytes of the type.

## fixed_set

Data structure describing a fixed set of size _n_.

## hash

A simple `Hashable` trait which is used to describe how an object should be hashed.

## hex

Functions for conversion between binary and hex string.

## locks

Macros for RwLock.

## message_format

A `MessageFormat` trait that handles conversion from and to binary, json, or base64.

## Hidden

A wrapper type for concealing sensitive information in logs.
# Feature Flags
Some of the utilities can be removed with feature flags:

## std

This will allow epoch_time, encoding and safe_array

## zero

This will include zeroize support

## serialize

This will include support for serde

## borsh

This will include support for borsh

## default

This will include all feature flags.