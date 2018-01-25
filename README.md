# Ergo: making rust's ecosystem more ergonomic, therefore more fun.

Ergo is an effort to unify the rust ecosystem at critical sections. It is
currently focused on improving the cli ergonomics. To accomplish this it will
create _multiple targeted conglomeration_ crates, each one with it's own
documentation and integration tests ensuring that the underlying libraries do
not break upstream users.

Unlike other conglomeration crates, ergo crates act as a _library_, where all
of its dependencies use the `>=version` form. This allows users to pin versions
of specific crates and otherwise stay at the cutting edge.

Some background:
- I blogged more about my intentions [here][blog]
- [This issue][qui_issue] contains a good deal of the intention as well

[blog]: http://vitiral.github.io/2018/01/17/rust2018-and-the-great-cli-awakening.html
[qui_issue]: https://github.com/killercup/quicli/issues/19
