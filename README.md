# Ergo: making rust's ecosystem more ergonomic, therefore more fun.

_The Ergo Ecosystem_ is an effort to unify the rust ecosystem at critical
sections. It is currently focused on improving Command Line Interface (CLI)
ergonomics. To accomplish this it will create _multiple targeted
conglomeration_ crates. These crates do much more than simply exporting the
API of their sub-crates. They implement wrapper types to unify them, as well
as have tested documentation to ensure they interopate together reliably.

@autron said it best in the
[rust 2018 roadmap](https://github.com/aturon/rfcs/blob/roadmap-2018/text/0000-roadmap-2018.md#cli-apps)

> ### CLI apps
> > Rust is a fantastic language for writing a Command Line Application (CLI).
> > For the ergonomics of hacking, it has one of the best argument parsers
> > ever, has seriously the best serialization library ever and it compiles to
> > almost any target and goes fast when it runs. (@vitiral)
>
> Rust has also seem some production update in the CLI space, for which it is
> very well-suited. This is a space where Rust’s portability, reliability, and
> ability to produce static binaries make it extremely attractive. We also have a
> number of excellent libraries already. This year, we will improve this
> ecosystem and pull it together into a polished, coherent package for people
> checking out Rust. Read @vitiral’s post and @killercup’s crate for some
> inspiration!


# Vision
Ergo's current goal is to be a full featuerd CLI SDK, built from composable
and distinct sub components. You should be able to depend on the `ergo` library
itself or each of its sub components individally.

Ergo aims to provide the following benefits:
- A standardized API for disparate types/approaches, allowing library authors
  to develop simple libraries of high quality, which can then be combined into
  an ecosystem with a unified API and excellent ergonomics.
- A starting point for CLI and application developers for documentation and
  *How To* guides. We hope to release an **Ergo Cookbook** once the libraries
  are more stable.
- Encourage interopability, quality, and ergonomic errors among the major CLI
  crates in the rust ecosystem and act as a driver towards higher quality and
  uniformity.


# Sub Crates
The ergo ecosystem is split into multiple crates, each with the prefix `ergo_`

The `ergo` crate itself is _currently in alpha status_. The primary author is
rewriting his CLI application using it to get the rough edges ironed out and
we are looking for feedback, contributors and leaders from the community
during this time

For now, consuder using [quicli](https://github.com/killercup/quicli) which
will [integrate cleanly with the ergo
ecosystem](https://github.com/killercup/quicli/issues/43) in the future.
If you _do_ use this crate expect frequenty changes that are not semver
compliant.


## Implemented Sub Crates
- [x] [**ergo_fs**](https://github.com/vitiral/ergo_fs): ergonomic filesystem
  operations. (_beta status_)
- [x] [**ergo_sync**](https://github.com/rust-crates/ergo_sync): provides an
  ultra-simple API for using `Sync` types, i.e. running threads and sending
  messages. (_beta status_)
- [x] [**ergo_std**](https://github.com/rust-crates/ergo_std): "generally
  needed stuff" -- `regex`, `lazy_static`, `maplit`, `itertools`, `ordermap`.
  This will be _very few crates_. It is mostly composed of things which could
  practically be in the std library. (_beta status_)
- [x] [**ergo_config**](https://github.com/rust-crates/ergo_config):
  deserialization and config files and ENV variables: `ron`, `toml`,
  `serde_json`, `serde_yaml`, `configure` (_alpha status_)
- [x] [**ergo_sys**](https://github.com/rust-crates/ergo_sys): deal with
  interfacing with the OS. Examples currently only include signal handling
  (`ctrlc`) and randomness (`rand`) but we are looking for other important
  crates. (_alpha status_)


## Future Sub Crates
- [ ] **ergo_client**: methods/types to be an HTTP client. Sub crates probably
  include `reqwest`, `h2` and some kinds of json-rpc+soap protocol helpers.
- [ ] **ergo_term**: simple and ergonomic terminal rendering.
- [ ] **ergo_test**: one-stop-shop for core testing functionality, mocking,
  etc.


# LICENSE
The source code in this repository is Licensed under either of
- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in the work by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
