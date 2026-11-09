# Rocket Ignite example

A Rocket app that shows Ignite as a dashboard and CRUD, with Propel views. It is the working example for [Ignite](https://github.com/lajennylove/Ignite).

Status: scaffold. The app renders one Propel view. Ignite is not wired in yet.

## Run

```sh
cargo run
```

Views are read from `resources/views/` on each request, as in the Propel docs. Release builds are refused until Propel's compiler exists (see Propel's `docs/production.md`).
