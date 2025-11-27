<!--
Guidance for AI coding assistants working on EuclidOLAP `olap-core`.
Keep this file concise and focused on patterns an AI needs to be productive.
-->

# Copilot instructions for olap-core

These notes explain the architecture, developer workflows, and project-specific
conventions that help an AI agent (or new contributor) make meaningful changes.

1. Big picture
- **Purpose:** `olap-core` is a Rust async gRPC server that parses MDX-like
  queries, builds a multidimensional context, and computes vectors returned
  over gRPC.
- **Major components:**
  - `src/main.rs` — entrypoint: starts the tonic gRPC server and dispatches
    requests to the MDX handler.
  - `src/exmdx/` — MDX parser/AST and execution helpers (AST types, functions,
    sets, expressions). Key files: `ast.rs`, `exp_func.rs`, `lv_func.rs`, `mem_func.rs`.
  - `src/mdx_grammar.lalrpop` and `src/mdx_lexer.rs` / `src/mdx_tokens.rs` —
    grammar + lexer. Modify `.lalrpop` or custom lexer rather than altering
    generated parser outputs directly.
  - `src/mdd.rs` and `src/exmdx/mdd.rs` — core multidimensional types
    (Cube, Axis, TupleVector, VectorValue) used by the calculation engine.
  - `src/calcul.rs` — calculation engine that evaluates coordinates into values.
  - `src/meta_cache.rs` and `src/cache.rs` — metadata caching and bootstrap
    (talks to external metadata service via gRPC client).
  - `proto/` and `build.rs` — protobuf definitions and build-time codegen.

2. Build & codegen workflows (explicit)
- Run the normal build to trigger all code generation steps (protos + LALRPOP):

  ```bash
  cargo build
  ```

- What `build.rs` does:
  - Compiles `proto/*.proto` with `tonic-build` into Cargo's `OUT_DIR`.
  - Calls `lalrpop::process_src()` to generate the parser code from
    `mdx_grammar.lalrpop`.

- Notes when changing proto or grammar:
  - Edit files under `proto/` for protobuf changes; then run `cargo build` to
    regenerate Rust types. Generated proto code goes into `$OUT_DIR` and is
    included at runtime via `tonic::include_proto!` (see `src/main.rs`).
  - Edit `src/mdx_grammar.lalrpop` (and `src/mdx_lexer.rs` for tokenization).
    Re-run `cargo build` so LALRPOP re-generates parser code. Do not edit
    `target/` or generated files directly.

3. Running & debugging
- Run the server locally:

  ```bash
  RUST_LOG=info cargo run --bin olap-core
  # or simply
  RUST_LOG=debug cargo run
  ```

- The server listens on `0.0.0.0:50052` by default (see `src/main.rs`).

4. Integration points & external dependencies
- gRPC services and protobufs: `proto/euclidolap.proto`, `proto/olapmeta.proto`,
  `proto/agg-service.proto`. The runtime uses `tonic`/`prost`.
- Metadata service: code uses a gRPC `GrpcClient` (see `src/olapmeta_grpc_client.rs`) to
  fetch cubes, members, levels and formula members. `meta_cache::init()` loads
  this data at startup.
- Parser tooling: LALRPOP (`mdx_grammar.lalrpop`) + `logos` custom lexer
  (see `src/mdx_lexer.rs`).

5. Project-specific conventions & patterns
- Async-first design: most APIs are `async`/`tokio` based; AST trait methods
  return boxed futures (`BoxFuture`) rather than using async-trait.
- Materialization model: AST nodes implement `Materializable` and
  `ToVectorValue` to convert AST fragments into runtime entities or values.
  Look at `src/exmdx/ast.rs` for usage patterns.
- Caching: global caches use `once_cell::sync::Lazy` + `Mutex`/`RwLock`. Be
  careful with blocking synchronous locks inside async code.
- Error/flow style: the code often uses `panic!` in unexpected cases. Prefer
  to follow existing error handling patterns when changing behavior.

6. Common change examples (concrete)
- Add an MDX function: update `src/exmdx/exp_func.rs` to add the function
  implementation, add AST variant if necessary, and ensure `ast.rs` maps
  parser-produced nodes into the new function type. Update grammar in
  `src/mdx_grammar.lalrpop` if the syntax changes, then `cargo build`.
- Add a protobuf field / RPC: edit `proto/*.proto`, then run `cargo build` to
  regenerate prost/tonic code. Update server/client code under `src/` to use
  the new types. Remember generated code lives in `OUT_DIR` and is included
  through `tonic::include_proto!` (check `src/main.rs`).

7. Files and code to avoid editing
- Do not edit files under `target/` or any `OUT_DIR` generated artifacts.
- Avoid changing generated parser outputs; instead modify `mdx_grammar.lalrpop`
  or the custom lexer `src/mdx_lexer.rs`.

8. Helpful references in this repository
- `src/main.rs` — entrypoint and gRPC wiring
- `build.rs` — proto + LALRPOP codegen
- `proto/` — protobufs for external API and metadata
- `src/exmdx/` — AST, functions, and MDX execution pipeline
- `src/calcul.rs`, `src/mdd.rs` — calculation core and data model
- `src/meta_cache.rs`, `src/olapmeta_grpc_client.rs` — metadata integration

If any part of this summary is unclear or you want extra examples (for
editing grammar, adding a new MDX function, or regenerating protos), tell me
which area to expand and I'll iterate.
