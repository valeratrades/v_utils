# v_utils
![Minimum Supported Rust Version](https://img.shields.io/badge/nightly-1.81+-ab6000.svg)
[<img alt="crates.io" src="https://img.shields.io/crates/v/v_utils.svg?color=fc8d62&logo=rust" height="20" style=flat-square>](https://crates.io/crates/v_utils)
[<img alt="docs.rs" src="https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs&style=flat-square" height="20">](https://docs.rs/v_utils)
[<img alt="build status" src="https://img.shields.io/github/actions/workflow/status/valeratrades/v_utils/ci.yml?branch=master&style=for-the-badge&style=flat-square" height="20">](https://github.com/valeratrades/v_utils/actions?query=branch%3Amaster) <!--NB: Won't find it if repo is private-->
[![Lines Of Code](https://tokei.rs/b1/github/valeratrades/v_utils?category=code)](https://github.com/valeratrades/v_utils/tree/master/src)

My utils crate. For personal use only. But maybe, just maybe, one day I will document it slightly more.
`v_utils` is optimized for **developer productivity**, other concerns, including even performance, are secondary. Iteration speed above all.

# Dev
## Testing
Some tests are hidden behind `slow_tests` feature-flag, so before release run tests with `-F slow_tests`
```sh
cargo nextest run -F slow_tests
```

<br>

<sup>
This repository follows <a href="https://github.com/valeratrades/.github/tree/master/best_practices">my best practices</a>.
</sup>

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
