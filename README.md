# Noir Differential Testing Framework

> this is a prototype library, do not use in production.

## Project Layout

- `circuits/`: noir circuit directory
- `src/`: rust directory
  - `lib.rs`: entry point + testing suite
  - `util.rs`: noir function exporter + circuit runner
- `Cargo.toml`: cargo config
- `Nargo.toml`: noir config

## Usage

```bash
cargo test
```

More explicitly:

```rs
fn main() {
    let input_map = BTreeMap::from([
        // -- snip
    ]);

    let thread_pool = rayon::ThreadPoolBuilder::new()
        .num_threads(1)
        .stack_size(4 * 1024 * 1024)
        .build()
        .unwrap();

    let result = thread_pool.install(|| {
        noir_fn("differential_vibe_check", input_map)
    });

    // -- snip
}
```

## Process

1. construct field elements in rust
2. compute input map
3. construct thread pool
4. run noir code on thread
   1. construct workspace from nargo toml config
   2. parse noir files
   3. get package from workspace
   4. compile exported functions
      1. check and report errors
      2. get exported functions
      3. compile each and report errors, if any
   5. find exported function by name
   6. compute initial witness without outputs
   7. execute acvm program solver
   8. extract and decode output from solved witness
5. check result against rust implementation

## Limitations

- must be `lib` package (blocked)
- recompiles on each test
- lots of github imports
- rudimentary error handling
- rust allocates a small stack for threads other than main, causing overflow

## Next Steps

- remove compilation steps, check that `nargo export` has been run first
- improve error handling
- breakout as generic rust crate for importing into other projects
  - macro-ize for library consumers
  - re-export relevant abi utilities
