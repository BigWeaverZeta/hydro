<a href="https://hydro.run"><h1 align="center">
    <img src="https://raw.githubusercontent.com/hydro-project/hydro/main/docs/static/img/hydro-logo.svg" width="400" alt='"hf"'>
</h1></a>
<p align="center">
    <a href="https://crates.io/crates/hydro_lang"><img src="https://img.shields.io/crates/v/hydro_lang?style=flat-square&logo=rust" alt="Crates.io"></a>
    <a href="https://docs.rs/hydro_lang/"><img src="https://img.shields.io/badge/docs.rs-Hydro-blue?style=flat-square&logo=read-the-docs&logoColor=white" alt="Docs.rs"></a>
    <a href="https://github.com/hydro-project/hydro/actions"><img src="https://img.shields.io/github/actions/workflow/status/hydro-project/hydro/ci.yml?style=flat-square&logo=github" alt="CI Status"></a>
    <a href="https://github.com/hydro-project/hydro"><img src="https://img.shields.io/badge/github-hydro--project%2Fhydro-8da0cb?style=flat-square&logo=github" alt="GitHub"></a>
    <a href="https://github.com/hydro-project/hydro/blob/main/LICENSE"><img src="https://img.shields.io/badge/license-Apache--2.0-blue?style=flat-square" alt="License"></a>
</p>

---

## Table of Contents

- [Overview](#overview)
- [Key Features](#key-features)
- [Architecture](#architecture)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Usage Examples](#usage-examples)
- [API and Key Components](#api-and-key-components)
- [Repository Structure](#repository-structure)
- [Documentation and Learning Resources](#documentation-and-learning-resources)
- [Contributing](#contributing)
- [Troubleshooting](#troubleshooting)
- [FAQ](#faq)
- [License](#license)
- [Citation](#citation)

---

## Overview

Hydro is a **high-performance distributed programming framework for Rust** that enables safer and more efficient distributed computing through advanced dataflow techniques, automatic vectorization, and built-in distributed safety mechanisms.

Much like Rust helps with memory safety, Hydro helps with [**distributed safety**](https://hydro.run/docs/hydro/correctness) -- using compile-time type checking to prevent common distributed systems bugs before they reach production.

Hydro integrates naturally with standard Rust constructs and IDEs, providing types and programming constructs for ensuring distributed safety. Under the covers, Hydro is powered by the **Dataflow Intermediate Representation (DFIR)**, a compiler and low-level runtime for stream processing that enables automatic vectorization and efficient scheduling without restricting your application logic.

**Get started today at [hydro.run](https://hydro.run)!**

---

## Key Features

### Distributed Safety

Hydro's type system catches distributed systems bugs at compile time:

- **Non-determinism prevention**: Eliminates bugs from message ordering, stream interleaving, and retries. Any intentional non-determinism must be explicitly annotated with the `nondet!()` macro.
- **Type-safe serialization**: Prevents mismatched serialization/deserialization formats across services by enforcing serialization boundaries through the type system (e.g., `send_bincode`, `send_bytes`).
- **Cluster identity safety**: Catches misuse of node identifiers across independent clusters at compile time, preventing accidental cross-cluster communication.
- **Deterministic batching**: Ensures predictable event batching without relying on wall-clock time, making behavior reproducible across different environments.

### High Performance

Built for speed with multiple optimization layers:

- **Automatic vectorization**: The DFIR compiler optimizes dataflow operations automatically, fusing operators where possible.
- **Rust monomorphization**: Zero-overhead abstractions through compile-time specialization -- all type parameters are resolved at compile time.
- **Two-layer architecture**: Combines the flexibility of scheduled dataflow with the efficiency of compiled operators (see [Architecture](#architecture)).
- **Single-threaded runtime support**: Reduced overhead with optional single-threaded Tokio execution for latency-sensitive workloads.
- **Query rewriting optimizations**: Automatic protocol optimizations via dataflow rewrites, inspired by database query optimization techniques.

### Advanced Dataflow Techniques

Powerful programming model for distributed systems:

- **Live Collections**: Dynamic, asynchronously-updated data sources including `Stream`, `Singleton`, `Optional`, `KeyedStream`, and `KeyedSingleton`.
- **Location-oriented programming**: Explicit computation placement across `Process` (single node), `Cluster` (group of identical nodes), `Tick` (iteration boundary), and `External` (client-facing interface) locations.
- **DFIR flow syntax**: Embeddable dataflow language with intuitive operator chaining using `->` syntax.
- **Iterative algorithms**: Native support for loops with iteration tracking via `Tick` boundaries.
- **Lattice-based computing**: CALM theorem support for coordination-free implementations using monotonic lattice operations.

### Multi-Cloud Deployment

Deploy anywhere with built-in orchestration:

- **Hydro Deploy**: Framework for launching programs on localhost, GCP, Azure, and more.
- **Two-stage compilation**: Generate deployment plans on your laptop, compile to machine-specific binaries on-the-fly.
- **Transparent networking**: Automatic serialization and network communication between locations -- you focus on logic, Hydro handles the plumbing.
- **Simulation support**: Test distributed protocols deterministically with the built-in simulator before deploying.

---

## Architecture

Hydro uses a unique **two-layer architecture** that combines the best of both static and dynamic dataflow models:

```
+----------------------------------------------------------+
|                    Your Application                       |
+----------------------------------------------------------+
|                     hydro_lang                            |
|           High-level API with type-safe locations         |
|     (Process, Cluster, Tick, Stream, Singleton, ...)      |
+----------------------------------------------------------+
|                      hydro_std                            |
|         Reusable distributed protocols & patterns         |
|           (Paxos, 2PC, quorum, broadcasts)                |
+----------------------------------------------------------+
|                      dfir_rs                              |
|         Dataflow Intermediate Representation Runtime      |
|    +--------------------------------------------------+   |
|    | Compiled Layer (inner)                            |   |
|    |   Based on Rust iterators, monomorphized for     |   |
|    |   zero-overhead performance                      |   |
|    +--------------------------------------------------+   |
|    | Scheduled Layer (outer)                           |   |
|    |   Timely-style top-down scheduling for complex   |   |
|    |   graph topologies and cyclic dataflows           |   |
|    +--------------------------------------------------+   |
+----------------------------------------------------------+
|                    hydro_deploy                           |
|   Multi-cloud orchestration (localhost, GCP, Azure, ...) |
+----------------------------------------------------------+
|                     lattices                              |
|   Algebraic lattice types for coordination-free CRDTs    |
+----------------------------------------------------------+
```

This hybrid approach provides both the flexibility of dynamic dataflow graphs and the performance of statically compiled code. The compiled layer handles linear chains of operators with iterator-like efficiency, while the scheduled layer manages complex graph structures, cycles, and dynamic scheduling.

---

## Installation

### Prerequisites

| Requirement | Version | Notes |
|-------------|---------|-------|
| **Rust** | 1.90.0+ | Automatically managed via `rust-toolchain.toml` |
| **Cargo** | (bundled) | Comes with Rust installation |
| **Python 3** | 3.10+ | Required for some deployment tooling |
| **Node.js** | Latest LTS | Only needed for Wasm/playground development |

### Adding Hydro to Your Project

Add Hydro to your `Cargo.toml`:

```toml
[dependencies]
hydro_lang = "0.14.0"

[build-dependencies]
hydro_build_utils = "0.0.1"
stageleft_tool = "0.10.0"
```

For deployment support, also add:

```toml
[dependencies]
hydro_deploy = "0.14.0"
tokio = "1.29.0"
```

### Using the DFIR Low-Level API

For direct DFIR programming without the high-level Hydro abstractions:

```toml
[dependencies]
dfir_rs = "0.14.0"
```

### Using Lattice Data Structures

For standalone lattice types (useful for CRDTs and coordination-free algorithms):

```toml
[dependencies]
lattices = "0.6.1"
```

### Creating a New Hydro Project

You can use the provided template to bootstrap a new project:

```bash
# Clone the repository
git clone https://github.com/hydro-project/hydro.git
cd hydro

# Use the template as a starting point
cp -r template/hydro my-hydro-project
cd my-hydro-project
```

There is also a DFIR-only template at `template/dfir/`.

---

## Quick Start

Here is a minimal end-to-end example to get you running with Hydro. This creates a single-process dataflow that generates and processes a stream of numbers:

```rust
use hydro_lang::prelude::*;

pub fn first_ten(process: &Process) {
    process
        .source_iter(q!(0..10))       // Generate numbers 0 through 9
        .map(q!(|n| n * 2))           // Double each number
        .for_each(q!(|n| println!("Result: {}", n)));
}
```

The `q!()` macro quotes Rust expressions for staged compilation -- this is how Hydro captures your logic to compile it into an optimized dataflow graph.

---

## Usage Examples

### Example 1: Distributed Computing Across a Cluster

This example distributes work from a leader process to a cluster of workers:

```rust
use hydro_lang::prelude::*;

pub struct Leader {}
pub struct Worker {}

pub fn distributed_computation<'a>(
    leader: &Process<'a, Leader>,
    workers: &Cluster<'a, Worker>
) {
    leader
        .source_iter(q!(0..100))
        // Distribute work across cluster using round-robin
        .round_robin_bincode(
            workers,
            nondet!(/** distribution order doesn't affect correctness */),
        )
        // Each worker processes independently
        .map(q!(|n| n * n))
        // Aggregate results back to leader
        .send_bincode(leader)
        .values()
        .assume_ordering(nondet!(/** intentional non-deterministic order */))
        .for_each(q!(|result| println!("Computed: {}", result)));
}
```

Key points:
- `nondet!()` explicitly marks places where non-determinism is acceptable
- `round_robin_bincode` distributes items across cluster members using bincode serialization
- `send_bincode` aggregates results back to a specific process

### Example 2: Low-Level DFIR Dataflow

For fine-grained control, you can use the DFIR flow syntax directly:

```rust
use dfir_rs::dfir_syntax;

fn main() {
    let (output_send, mut output_recv) = dfir_rs::util::unbounded_channel::<char>();

    let mut flow = dfir_syntax! {
        source_iter("Hello World".chars())
            -> map(|c| c.to_ascii_uppercase())
            -> for_each(|c| output_send.send(c).unwrap());
    };

    flow.run_available_sync();

    let output = &*dfir_rs::util::collect_ready::<String, _>(&mut output_recv);
    assert_eq!(output, "HELLO WORLD");
}
```

### Example 3: Multi-Cloud Deployment

Deploy your Hydro programs to cloud infrastructure:

```rust
use hydro_deploy::{Deployment, Host};
use hydro_deploy::gcp::GcpNetwork;
use hydro_lang::deploy::TrybuildHost;
use std::sync::Arc;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() {
    let mut deployment = Deployment::new();

    // Configure GCP deployment
    let project = "my-gcp-project";
    let network = Arc::new(RwLock::new(GcpNetwork::new(project, None)));

    let host = deployment
        .GcpComputeEngineHost()
        .project(project)
        .machine_type("e2-micro")
        .image("debian-cloud/debian-11")
        .region("us-west1-a")
        .network(network)
        .add();

    // Build and deploy your Hydro program
    let builder = hydro_lang::compile::builder::FlowBuilder::new();
    let process = builder.process();

    // Define your dataflow logic here
    first_ten(&process);

    // Deploy and run
    let _nodes = builder
        .with_process(&process, TrybuildHost::new(host))
        .deploy(&mut deployment);

    deployment.run_ctrl_c().await.unwrap();
}
```

### Example 4: Using Lattices for CRDTs

Lattice types enable conflict-free replicated data types:

```rust
use lattices::{Max, Merge};

let mut a = Max::new(3);
let b = Max::new(5);

// Merge is commutative, associative, and idempotent
a.merge(b);
assert_eq!(a, Max::new(5));
```

### More Examples

The repository includes many real-world distributed protocol implementations:

| Example | Location | Description |
|---------|----------|-------------|
| **Paxos** | `hydro_test/src/cluster/paxos.rs` | Classic consensus protocol |
| **Two-Phase Commit** | `hydro_test/src/cluster/two_pc.rs` | Distributed transaction protocol |
| **MapReduce** | `hydro_test/src/cluster/map_reduce.rs` | Parallel data processing pattern |
| **Chat Application** | `hydro_test/src/local/chat_app.rs` | Local multi-user chat |
| **Graph Reachability** | `hydro_test/src/local/graph_reachability.rs` | Graph algorithm example |
| **KV Store** | `dfir_rs/examples/kvs/` | Key-value store with DFIR |
| **Replicated KV Store** | `dfir_rs/examples/kvs_replicated/` | Replicated key-value store |
| **Echo Server** | `dfir_rs/examples/echo_server/` | Simple echo server |
| **Chat Server** | `dfir_rs/examples/chat/` | Chat server with DFIR |
| **Vector Clocks** | `dfir_rs/examples/lamport_clock/` | Lamport/vector clock implementation |
| **Compute Pi** | `hydro_test/src/cluster/compute_pi.rs` | Distributed Monte Carlo Pi estimation |

---

## API and Key Components

### Core Libraries

#### `hydro_lang` -- High-Level Framework
[![crates.io](https://img.shields.io/crates/v/hydro_lang?style=flat-square)](https://crates.io/crates/hydro_lang)
[![docs.rs](https://img.shields.io/badge/docs.rs-hydro__lang-blue?style=flat-square)](https://docs.rs/hydro_lang/)

The main entry point for distributed programming with type-safe location abstractions:

- **Locations**: `Process`, `Cluster`, `Tick`, `External` -- explicit computation placement
- **Live Collections**: `Stream`, `Singleton`, `Optional`, `KeyedStream`, `KeyedSingleton` -- reactive data sources
- **Operators**: `map`, `filter`, `fold`, `reduce`, `join`, `cross_product`, `send_bincode`, `send_bytes`, `round_robin_bincode`, and more
- **Safety Guards**: `nondet!()` macro for explicitly marking intentional non-determinism

[Hydro Lang Documentation](https://hydro.run/docs/hydro/)

#### `dfir_rs` -- Dataflow Runtime
[![crates.io](https://img.shields.io/crates/v/dfir_rs?style=flat-square)](https://crates.io/crates/dfir_rs)
[![docs.rs](https://img.shields.io/badge/docs.rs-dfir__rs-blue?style=flat-square)](https://docs.rs/dfir_rs/)

Low-level dataflow compiler and runtime with the DFIR flow syntax:

- **Flow Syntax**: Embeddable dataflow language with `->` operator chaining
- **Operators**: `source_iter`, `source_stream`, `map`, `filter`, `fold`, `join`, `tee`, `union`, `unique`, and many more
- **Runtime**: Scheduled execution with batching and low-latency processing
- **Graph Visualization**: Built-in Mermaid and DOT export for dataflow graphs

[DFIR Documentation](https://hydro.run/docs/dfir/) | [DFIR Playground](https://hydro.run/docs/dfir/playground)

#### `hydro_deploy` -- Deployment Framework
[![crates.io](https://img.shields.io/crates/v/hydro_deploy?style=flat-square)](https://crates.io/crates/hydro_deploy)
[![docs.rs](https://img.shields.io/badge/docs.rs-hydro__deploy-blue?style=flat-square)](https://docs.rs/hydro_deploy/)

Multi-cloud orchestration for Hydro programs:

- **Localhost**: Development and testing on local machines
- **GCP**: Google Cloud Platform Compute Engine deployment
- **Azure**: Azure VM deployment
- **Custom Hosts**: Extensible host interface for any platform
- **Transparent Networking**: Automatic connection setup between distributed processes

[Hydro Deploy Documentation](https://hydro.run/docs/deploy/)

#### `lattices` -- Lattice Data Structures
[![crates.io](https://img.shields.io/crates/v/lattices?style=flat-square)](https://crates.io/crates/lattices)
[![docs.rs](https://img.shields.io/badge/docs.rs-lattices-blue?style=flat-square)](https://docs.rs/lattices/)

Mathematical lattices for coordination-free distributed computing:

- **Built-in Lattices**: `Min`, `Max`, `SetUnion`, `MapUnion`, `UnionFind`, `VecUnion`
- **Composition**: `WithBot`, `WithTop`, `Pair`, `DomPair` for nested lattices
- **CALM Theorem**: Monotonic lattice operations enable coordination-free distribution
- **CRDTs**: Conflict-free Replicated Data Types via lattice merge operations

[Lattices Documentation](https://hydro.run/docs/dfir/lattices_crate/)

#### `hydro_std` -- Standard Library
[![crates.io](https://img.shields.io/crates/v/hydro_std?style=flat-square)](https://crates.io/crates/hydro_std)

Reusable distributed protocols and patterns built with Hydro, including quorum logic, broadcast utilities, and common distributed building blocks.

### Supporting Libraries

| Crate | Description |
|-------|-------------|
| `dfir_lang` | DFIR intermediate representation compiler |
| `dfir_macro` | Procedural macro for `dfir_syntax!` |
| `hydro_build_utils` | Build-time utilities shared across Hydro crates |
| `hydro_deploy_integration` | Runtime communication layer for deployed processes |
| `variadics` | Variadic generics emulation via tuple lists |
| `variadics_macro` | Procedural macros for `variadics` |
| `lattices_macro` | Procedural macros for `lattices` |
| `sinktools` | Push-based sink utilities (e.g., `DemuxMap`) |
| `multiplatform_test` | Cross-platform test macro (native + Wasm) |
| `include_mdtests` | Markdown-embedded test extraction |

---

## Repository Structure

```
hydro/
├── hydro_lang/             # High-level Hydro framework (main entry point)
├── hydro_std/              # Standard library of distributed protocols
├── hydro_test/             # Integration tests and example protocols
│   ├── src/cluster/        #   Cluster protocols: Paxos, 2PC, MapReduce, ...
│   ├── src/distributed/    #   Multi-node distributed tests
│   ├── src/local/          #   Single-node tests: chat, graph reachability, ...
│   └── src/external_client/#   External client interface tests
├── hydro_deploy/           # Multi-cloud deployment framework
│   ├── core/               #   Core deployment library
│   ├── hydro_deploy_integration/ # Runtime communication layer
│   └── hydro_deploy_examples/    # Deployment examples
├── dfir_rs/                # DFIR runtime and compiled/scheduled layers
│   ├── src/                #   Runtime, scheduler, and operator implementations
│   ├── examples/           #   DFIR examples (chat, KVS, echo, ...)
│   └── tests/              #   Unit and snapshot tests
├── dfir_lang/              # DFIR intermediate representation compiler
├── dfir_macro/             # Procedural macro for dfir_syntax!
├── lattices/               # Lattice data structures library
├── lattices_macro/         # Lattice procedural macros
├── variadics/              # Variadic generics emulation
├── variadics_macro/        # Variadics procedural macros
├── sinktools/              # Push-based sink utilities
├── hydro_build_utils/      # Shared build utilities
├── multiplatform_test/     # Cross-platform test macro
├── include_mdtests/        # Markdown-embedded test support
├── benches/                # Microbenchmarks
├── docs/                   # hydro.run website source (Docusaurus)
├── website_playground/     # DFIR in-browser playground (Wasm)
├── template/               # Project templates
│   ├── hydro/              #   Hydro project template
│   └── dfir/               #   DFIR project template
├── design_docs/            # Historical architecture design documents
├── scripts/                # Build and release scripts
├── Cargo.toml              # Workspace configuration
├── Cargo.lock              # Dependency lock file
├── rust-toolchain.toml     # Pinned Rust toolchain (1.90.0)
├── Dockerfile              # Container build configuration
├── CONTRIBUTING.md         # Contribution guidelines
├── RELEASING.md            # Release process documentation
├── CITATION.cff            # Academic citation metadata
└── LICENSE                 # Apache License 2.0
```

---

## Documentation and Learning Resources

### Official Documentation

| Resource | Description |
|----------|-------------|
| **[hydro.run](https://hydro.run)** | Main documentation hub |
| **[Hydro Framework Docs](https://hydro.run/docs/hydro/)** | High-level API, concepts, and tutorials |
| **[DFIR Docs](https://hydro.run/docs/dfir)** | Low-level dataflow programming guide |
| **[Deploy Docs](https://hydro.run/docs/deploy/)** | Multi-cloud deployment guide |
| **[API Reference](https://docs.rs/hydro_lang/)** | Rust API documentation (docs.rs) |
| **[DFIR Playground](https://hydro.run/docs/dfir/playground)** | Try DFIR in your browser |

### Concepts and Theory

- **[Distributed Safety](https://hydro.run/docs/hydro/correctness)** -- How Hydro prevents distributed systems bugs
- **[Lattices](https://hydro.run/docs/dfir/lattices_crate/)** -- Using lattice types for coordination-free computing
- **[CALM Theorem](https://cacm.acm.org/research/keeping-calm/)** -- Theoretical foundation for consistency without coordination

### Research Papers

Hydro is backed by peer-reviewed research from UC Berkeley:

| Paper | Venue | Topic |
|-------|-------|-------|
| [New Directions in Cloud Programming](https://hydro.run/papers/new-directions.pdf) | CIDR 2021 | Original Hydro vision |
| [Optimizing Distributed Protocols with Query Rewrites](https://hydro.run/papers/david-sigmod-2024.pdf) | SIGMOD 2024 | Performance optimizations |
| [Flo: a Semantic Foundation for Progressive Stream Processing](https://arxiv.org/abs/2411.08274) | POPL 2025 | Formal semantics |

See [hydro.run/research](https://hydro.run/research) for the full list of publications.

---

## Contributing

We welcome contributions to Hydro! This is an experimental, research-driven project with opportunities for impactful contributions.

### Getting Started

1. **Fork and clone** the repository:
   ```bash
   git clone https://github.com/<your-username>/hydro.git
   cd hydro
   ```
2. **Rust toolchain** is managed automatically -- `rust-toolchain.toml` pins version 1.90.0.
3. **Build the project**:
   ```bash
   cargo build
   ```
4. **Run tests**:
   ```bash
   cargo test
   ```
5. **Run formatting and lint checks**:
   ```bash
   ./precheck.bash --all
   ```

### Development Workflow

- Follow [Conventional Commits](https://www.conventionalcommits.org/) for PR titles (e.g., `feat(hydro_lang): add new operator`)
- Run `./precheck.bash` before submitting PRs (supports `--dfir`, `--hydro`, `--all` flags)
- Update snapshot tests with `cargo insta review`
- Feature branches should be created off `main`:
  ```bash
  git checkout -b feature/my-feature origin/main
  ```

### Snapshot Testing

Hydro uses two snapshot testing frameworks:
- **[`insta`](https://insta.rs/)** -- General snapshot testing for generated DFIR graphs
- **[`trybuild`](https://github.com/dtolnay/trybuild)** -- Compile-error message testing

```bash
cargo install cargo-insta
cargo insta test
cargo insta review
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines.

---

## Troubleshooting

### Common Issues

#### Rust version mismatch
```
error: the option `Z` is only accepted on the nightly compiler
```
**Solution**: The project uses `rust-toolchain.toml` to automatically select the right Rust version. Make sure `rustup` is installed and let it manage the toolchain:
```bash
rustup show
```

#### Snapshot test failures after code changes
```
assertion failed: snapshot matches
```
**Solution**: Review and accept the new snapshots:
```bash
cargo insta test
cargo insta review
```

#### Wasm build errors
```
error: target `wasm32-unknown-unknown` not found
```
**Solution**: Install the required Wasm tooling:
```bash
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
```

#### Long compile times
**Solution**: For development, consider testing individual crates:
```bash
cargo test -p hydro_lang        # Test only hydro_lang
cargo test -p dfir_rs           # Test only dfir_rs
./precheck.bash --dfir          # Check only DFIR crates
./precheck.bash --hydro         # Check only Hydro crates
```

#### Docker build issues
The repository includes a multi-stage `Dockerfile` for building optimized release binaries:
```bash
docker build -t hydro .
```

---

## FAQ

**Q: What is the difference between Hydro and DFIR?**

Hydro (`hydro_lang`) is the high-level framework that provides type-safe distributed programming with location abstractions, safety checks, and deployment support. DFIR (`dfir_rs`) is the lower-level dataflow runtime that Hydro compiles down to. You can use DFIR directly for fine-grained control, or use Hydro for a higher-level experience. Think of it like the relationship between a high-level language and its compiler backend.

**Q: Do I need to understand DFIR to use Hydro?**

No. Hydro provides a complete high-level API. However, understanding DFIR can help with debugging and performance tuning, much like understanding assembly can help with C/C++ optimization.

**Q: What does `q!()` do?**

The `q!()` macro is part of Hydro's staged programming model (via the `stageleft` crate). It quotes Rust expressions so they can be captured at compile time and compiled into the dataflow graph, rather than being executed immediately. This enables Hydro's compile-time optimizations while keeping a natural Rust syntax.

**Q: What does `nondet!()` do?**

The `nondet!()` macro is a safety annotation that explicitly marks places where non-determinism is acceptable. Hydro's type system requires you to acknowledge any source of non-determinism (like message ordering) so that bugs from accidental non-determinism are caught at compile time.

**Q: Can I deploy to Kubernetes?**

Hydro Deploy currently supports localhost, GCP Compute Engine, and Azure VMs. Kubernetes support is not built in but the `Host` trait can be extended to support custom deployment targets.

**Q: Is Hydro production-ready?**

Hydro is an active research project from UC Berkeley. While it includes production-grade features like cloud deployment and comprehensive testing, it is primarily used in research settings. APIs may change between versions.

---

## License

Hydro is licensed under the **Apache License 2.0**.

See [LICENSE](LICENSE) for the full license text.

---

## Citation

If you use Hydro in academic work, please cite:

```bibtex
@inproceedings{hydro2021,
  title={New Directions in Cloud Programming},
  author={Hellerstein, Joseph M. and Faleiro, Jose M. and Gonzalez, Joseph E. and Schleier-Smith, Johann and Sreekanti, Vikram and Tumanov, Alexey and Wu, Chenggang},
  booktitle={CIDR},
  year={2021}
}
```

See [CITATION.cff](CITATION.cff) for additional citation formats and the full list of contributors.

---

<p align="center">
Built by the <a href="https://hydro.run">Hydro Project</a> at UC Berkeley
</p>
