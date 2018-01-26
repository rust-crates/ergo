# Ergo: making rust's ecosystem more ergonomic, therefore more fun.

Ergo is an effort to unify the rust ecosystem at critical sections. It is
currently focused on improving the cli ergonomics. To accomplish this it will
create _multiple targeted conglomeration_ crates, each one with it's own
documentation and integration tests ensuring that the underlying libraries do
not break upstream users.

## Sub Crates
This crate is not currently usable.

The following sub crates are targeted towards the following months:

- [ ]: **ergo_base**: "generally needed stuff" -- `lazy_static`, `maplit`, `failure`, `itertools`, `std_prelude`, etc.
- [ ]: **ergo_sys**: deal with interfacing with the OS. Examples include time
  (`chrono`), signal handling `ctrlc`, (maybe) system libraries (`libc`), shell
  variables (`shellexpand`) and randomness (`rand`).
- [x]: ~~**ergo_fs** for files+directories. Some crates could be `path_abs`,
  `walkdir`, `tar`~~ [repo](https://github.com/vitiral/ergo_fs)
- [ ]: **ergo_config** deserialization and config files+env-vars: `toml`, `serde_json`, `serde_yaml`, `configure`, etc

Something that the ecosystem isn't quite ready for but is probably close
- **ergo_term**: terminal input/output styling: `tabwriter`, `pretty_tables`, `termstyle`, etc
- **ergo_test**: test framework conglomeration, not particular to any particular application

## Background
- I blogged more about my intentions [here][blog]
- [This issue][qui_issue] contains a good deal of the intention as well


[blog]: http://vitiral.github.io/2018/01/17/rust2018-and-the-great-cli-awakening.html
[qui_issue]: https://github.com/killercup/quicli/issues/19
