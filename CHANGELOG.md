# Changelog

All notable changes to this project will be documented in this file. See [standard-version](https://github.com/conventional-changelog/standard-version) for commit guidelines.

### [0.4.9](https://github.com/tari-project/tari_utilities/compare/v0.4.8...v0.4.9) (2022-11-18)


### Features

* improve hidden data handling ([#52](https://github.com/tari-project/tari_utilities/issues/52)) ([f35006f](https://github.com/tari-project/tari_utilities/commit/f35006f7179533b7ac3a7a9ee1d6932475b95cec))

### [0.4.8](https://github.com/tari-project/tari_utilities/compare/v0.4.7...v0.4.8) (2022-10-05)


### Features

* add reveal method to Hidden ([#49](https://github.com/tari-project/tari_utilities/issues/49)) ([ee84d4f](https://github.com/tari-project/tari_utilities/commit/ee84d4fd9faa0ca21c9ec5e059612fc7aa106059))

### [0.4.7](https://github.com/tari-project/tari_utilities/compare/v0.4.6...v0.4.7) (2022-10-03)

### [0.4.6](https://github.com/tari-project/tari_utilities/compare/v0.4.5...v0.4.6) (2022-08-29)


### Bug Fixes

* a slightly changed signature of `impl From<String> for SafePassword` to be a bit more generic ([#47](https://github.com/tari-project/tari_utilities/issues/47)) ([3fba4df](https://github.com/tari-project/tari_utilities/commit/3fba4dfd441bd3df42b2fdb0e9e1db04daab6441))

### [0.4.5](https://github.com/tari-project/tari_utilities/compare/v0.4.4...v0.4.5) (2022-07-26)


### Features

* add SafePassword struct ([#46](https://github.com/tari-project/tari_utilities/issues/46)) ([ae01aeb](https://github.com/tari-project/tari_utilities/commit/ae01aeb27317575cd7066ed8591a4b9c93bac524))

### [0.4.4](https://github.com/tari-project/tari_utilities/compare/v0.4.3...v0.4.4) (2022-06-14)


### Features

* added `hex` module with a serializer and deserializer [94c1452](https://github.com/tari-project/tari_utilities/commit/94c1452c64bebc74733c43c92cd9b4fb3651ab02)
* `ByteArray` trait is implemented for byte arrays of all sizes [7f424dd](https://github.com/tari-project/tari_utilities/commit/7f424ddbc234b62f1564cc91e79692a095d32463)
* added `Hidden` wrapper [f1010ba](https://github.com/tari-project/tari_utilities/commit/f1010bab437c74941d0680b21d6f95fd9d10cc8c)

### [0.4.3](https://github.com/tari-project/tari_utilities/compare/v0.4.2...v0.4.3) (2022-04-29)


### Bug Fixes

* reduce serde_json dependency ([4e64c76](https://github.com/tari-project/tari_utilities/commit/4e64c76adccc9692099f5371129d136964fa7194))
* reduce thiserror dependency ([cc0f518](https://github.com/tari-project/tari_utilities/commit/cc0f518e47b8b01cad3c2628986904a46bd59aac))

### [0.4.2](https://github.com/tari-project/tari_utilities/compare/v0.4.1...v0.4.2) (2022-04-12)


### Bug Fixes

* reduce min serde version ([dbbbca0](https://github.com/tari-project/tari_utilities/commit/dbbbca0f2de3c6b1c1bef71eafa29de857c0a1a8))

### [0.4.1](https://github.com/tari-project/tari_utilities/compare/v0.4.0...v0.4.1) (2022-04-12)

* remove unused dependencies

## [0.4.0](https://github.com/tari-project/tari_utilities/compare/v0.3.1...v0.4.0) (2022-04-06)


### ⚠ BREAKING CHANGES

* remove chrono dependency, remove extend bytes (#25)

### Features

* uses standard version ([#29](https://github.com/tari-project/tari_utilities/issues/29)) ([7e0c6a0](https://github.com/tari-project/tari_utilities/commit/7e0c6a08b233be21709cba0436b1eff090c14f4a))


* remove chrono dependency, remove extend bytes ([#25](https://github.com/tari-project/tari_utilities/issues/25)) ([dbafd78](https://github.com/tari-project/tari_utilities/commit/dbafd7827896db3365c5a13e6823c96e4c941ef6)), closes [tari-project/tari-crypto#64](https://github.com/tari-project/tari-crypto/issues/64)

### [0.3.1](https://github.com/tari-project/tari_utilities/compare/v0.3.0...v0.3.1) (2022-03-11)


### Features

* add time crate support ([#15](https://github.com/tari-project/tari_utilities/issues/15)) ([635abfb](https://github.com/tari-project/tari_utilities/commit/635abfba9597a62e7a5dd486f90292724176f9a7))

### Bug Fixes

* add clock feature to chrono ([#17](https://github.com/tari-project/tari_utilities/issues/17)) ([f8046bc](https://github.com/tari-project/tari_utilities/commit/f8046bce7a995c63cc298f214330f0c992cf6fc6))
* avoid implicit time dependency ([#16](https://github.com/tari-project/tari_utilities/issues/16)) ([d9bd1f9](https://github.com/tari-project/tari_utilities/commit/d9bd1f9924eb180866f6b0b86309d945c33bf620))
* **byte_array:** remove unnecessary allocation in to_hex ([#19](https://github.com/tari-project/tari_utilities/issues/19)) ([b1ba343](https://github.com/tari-project/tari_utilities/commit/b1ba3438e48fc8f4136a6857e38ea0e32af5ecb1))
* source coverage is broken with latest nightly ([5a1e4b1](https://github.com/tari-project/tari_utilities/commit/5a1e4b1ee58f67ed199a8cfaa4369bad286b4f91))
