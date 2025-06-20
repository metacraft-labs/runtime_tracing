## runtime_tracing

A format and helper library for [CodeTracer](https://github.com/metacraft-labs/CodeTracer.git) traces

### format

A CodeTracer trace for its db backend consists of record data, metadata for the recorded program and copy of the relevant source/repository files. 
It's self contained, so one can debug a record of a version of a program from a different commit or branch. 
The record data consists of a stream of objects, each of which describes a certain program event: call, step, return etc.

A trace contains several files:
* `trace.json` – the event data in JSON format
* `trace.bin` – the same event stream encoded in a binary format (see
  [docs/trace_binary_spec.md](docs/trace_binary_spec.md))
* `trace_metadata.json` – metadata for the recorded program
* `trace_paths.json` – a list of the recorded files
* `files/` – a folder including all the source/repository files copied into the trace.

The event stream can be stored either as JSON (`trace.json`) or in the
binary format (`trace.bin`). Both representations correspond to the Rust
types in `src/types.rs`.

We plan on 
* defining a more precise document or a list of examples or a specification.
* producing a more optimized version of the format. A binary variant is now
  available as `trace.bin`.

A future goal of this format is to make it possible to stream traces: to be able to replay them while they're still being recorded. 
This is one of the reasons for the decision to maintain a single "stream" of events currently. 

### tracer library

We also define a Rust tracer library in `src/tracer.rs` which can be used as a helper to instrument Rust-based language interpreters and vm-s. 
It can make it easier to migrate to newer versions of the format, hiding many details behind its helpers. 
There are a few examples and tests of its usage in `src/lib.rs`. 

There are some actual usages of it as well which can be also used as an example:
* in [blocksense-network/noir: their tracing support for CodeTracer](https://github.com/blocksense-network/noir/tree/blocksense/tooling/tracer)
* in a small toy interpreter to be released as a part of the CodeTracer repo

One can always directly produce the same traces from various languages. We're open for cooperation or discussion on usecases!

### Building the Documentation

The library API docs can be built locally with:

```bash
cargo doc --all-features --no-deps
```

The generated HTML documentation will be placed under `target/doc`. Open
`target/doc/runtime_tracing/index.html` in a browser to inspect it.

### Publishing to crates.io

After updating the version number in `Cargo.toml`, publish a new release with:

```bash
cargo publish
```

Documentation for released versions will be automatically hosted on
[docs.rs](https://docs.rs/runtime_tracing).

### Legal

Authored and maintained by Metacraft Labs, Ltd

LICENSE: MIT
