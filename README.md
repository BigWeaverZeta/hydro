<a href="https://hydro.run"><h1 align="center">
    <img src="https://raw.githubusercontent.com/hydro-project/hydro/main/docs/static/img/hydro-logo.svg" width="400" alt='"hf"'>
</h1></a>
<p align="center">
    <a href="https://crates.io/crates/hydro_lang"><img src="https://img.shields.io/crates/v/hydro_lang?style=flat-square&logo=rust" alt="Crates.io"></a>
    <a href="https://docs.rs/hydro_lang/"><img src="https://img.shields.io/badge/docs.rs-Hydro-blue?style=flat-square&logo=read-the-docs&logoColor=white" alt="Docs.rs"></a>
    <a href="https://github.com/hydro-project/hydro"><img src="https://img.shields.io/badge/github-hydro--project%2Fhydro-8da0cb?style=flat-square&logo=github" alt="GitHub"></a>
</p>

# Overview

Hydro is a **high-performance distributed programming framework for Rust** that enables safer and more efficient distributed computing through advanced dataflow techniques, automatic vectorization, and built-in distributed safety mechanisms.

Much like Rust helps with memory safety, Hydro helps with [**distributed safety**](https://hydro.run/docs/hydro/correctness)—using compile-time type checking to prevent common distributed systems bugs before they reach production.

Hydro integrates naturally with standard Rust constructs and IDEs, providing types and programming constructs for ensuring distributed safety.
Under the covers, Hydro is powered by the **Dataflow Intermediate Representation (DFIR)**, a compiler and low-level runtime for stream processing that enables automatic vectorization and efficient scheduling without restricting your application logic.

**Get started today at [hydro.run](https://hydro.run)!**

## Key Features

### 🛡️ Distributed Safety
Hydro's type system catches distributed systems bugs at compile time:
- **Non-determinism prevention**: Eliminates bugs from message ordering, stream interleaving, and retries
- **Type-safe serialization**: Prevents mismatched serialization/deserialization formats across services
- **Cluster identity safety**: Catches misuse of node identifiers across independent clusters
- **Deterministic batching**: Ensures predictable event batching without relying on wall-clock time

### ⚡ High Performance
Built for speed with multiple optimization layers:
- **Automatic vectorization**: DFIR compiler optimizes dataflow operations automatically
- **Rust monomorphization**: Zero-overhead abstractions through compile-time specialization
- **Two-layer architecture**: Combines flexibility of scheduled dataflow with efficiency of compiled operators
- **Single-threaded runtime support**: Reduced overhead with optional single-threaded Tokio execution
- **Query rewriting optimizations**: Automatic protocol optimizations via dataflow rewrites

### 🌊 Advanced Dataflow Techniques
Powerful programming model for distributed systems:
- **Live Collections**: Dynamic, asynchronously-updated data sources (`Stream`, `Singleton`, `Optional`, `KeyedStream`)
- **Location-oriented programming**: Explicit computation placement across `Process`, `Cluster`, `Tick`, and `External` locations
- **DFIR flow syntax**: Embeddable dataflow language with intuitive operator chaining
- **Iterative algorithms**: Native support for loops with iteration tracking
- **Lattice-based computing**: CALM theorem support for coordination-free implementations

### 🚀 Multi-Cloud Deployment
Deploy anywhere with built-in orchestration:
- **Hydro Deploy**: Framework for launching programs on localhost, GCP, Azure, and more
- **Two-stage compilation**: Generate deployment plans on your laptop, compile to machine-specific binaries
- **Transparent networking**: Automatic serialization and network communication between locations

## Installation

### Prerequisites

- **Rust**: Version 1.90.0 or later (automatically managed via `rust-toolchain.toml`)
- **Cargo**: Comes with Rust installation

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

For direct DFIR programming:

```toml
[dependencies]
dfir_rs = "0.14.0"
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

## Usage Examples

### Example 1: Simple Stream Processing

```rust
use hydro_lang::prelude::*;

pub fn first_ten(process: &Process) {
    process
        .source_iter(q!(0..10))
        .map(q!(|n| n * 2))
        .for_each(q!(|n| println!("Result: {}", n)));
}
```

### Example 2: Distributed Computing Across a Cluster

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

### Example 3: Low-Level DFIR Dataflow

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

### Example 4: Multi-Cloud Deployment

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

## API and Key Components

### Core Libraries

#### `hydro_lang` - High-Level Framework
The main entry point for distributed programming with type-safe location abstractions:

- **Locations**: `Process`, `Cluster`, `Tick`, `External` - explicit computation placement
- **Live Collections**: `Stream`, `Singleton`, `Optional`, `KeyedStream`, `KeyedSingleton` - reactive data sources
- **Operators**: `map`, `filter`, `fold`, `reduce`, `join`, `cross_product`, `send_bincode`, and more
- **Safety Guards**: `nondet!()` macro for explicitly marking intentional non-determinism

📚 [Hydro Lang Documentation](https://hydro.run/docs/hydro/)

#### `dfir_rs` - Dataflow Runtime
Low-level dataflow compiler and runtime with the DFIR flow syntax:

- **Flow Syntax**: Embeddable dataflow language with operator chaining
- **Operators**: `source_iter`, `source_stream`, `map`, `filter`, `fold`, `join`, `tee`, and more
- **Runtime**: Scheduled execution with batching and low-latency processing
- **Graph Visualization**: Built-in Mermaid and DOT export for dataflow graphs

📚 [DFIR Documentation](https://hydro.run/docs/dfir/) | [DFIR Playground](https://hydro.run/docs/dfir/playground)

#### `hydro_deploy` - Deployment Framework
Multi-cloud orchestration for Hydro programs:

- **Localhost**: Development and testing on local machines
- **GCP**: Google Cloud Platform Compute Engine deployment
- **Azure**: Azure VM deployment
- **Custom Hosts**: Extensible host interface for any platform

📚 [Hydro Deploy Documentation](https://hydro.run/docs/deploy/)

#### `lattices` - Lattice Data Structures
Mathematical lattices for coordination-free distributed computing:

- **Built-in Lattices**: `Min`, `Max`, `SetUnion`, `MapUnion`, `UnionFind`, `VecUnion`
- **Composition**: `WithBot`, `WithTop`, `Pair`, `DomPair` for nested lattices
- **CALM Theorem**: Monotonic lattice operations enable coordination-free distribution
- **CRDTs**: Conflict-free Replicated Data Types via lattice merge

📚 [Lattices Documentation](https://hydro.run/docs/dfir/lattices_crate/)

#### `hydro_std` - Standard Library
Reusable distributed protocols and patterns built with Hydro.

### Architecture

Hydro uses a unique **two-layer architecture**:

1. **Compiled Layer** (inner): Based on Rust iterators, monomorphized for zero-overhead performance
2. **Scheduled Layer** (outer): Timely-style dataflow with top-down scheduling for complex graphs

This hybrid approach provides both the flexibility of dynamic dataflow graphs and the performance of statically compiled code.

## Documentation and Learning Resources

### Official Documentation
- **[Hydro Website](https://hydro.run)** - Main documentation hub
- **[Hydro Framework Docs](https://hydro.run/docs/hydro/)** - High-level API and concepts
- **[DFIR Docs](https://hydro.run/docs/dfir)** - Low-level dataflow programming
- **[API Reference](https://docs.rs/hydro_lang/)** - Rust API documentation

### Examples
- **`hydro_test/examples/`** - Distributed protocols (Paxos, Two-Phase Commit, MapReduce)
- **`dfir_rs/examples/`** - DFIR examples (chat server, key-value stores, vector clocks)
- **`template/hydro/`** - Project templates for getting started

### Research Papers
Our [research publications](https://hydro.run/research) provide theoretical foundations:
- [New Directions in Cloud Programming](https://hydro.run/papers/new-directions.pdf) (CIDR 2021) - Original vision
- [Optimizing Distributed Protocols with Query Rewrites](https://hydro.run/papers/david-sigmod-2024.pdf) (SIGMOD 2024) - Performance optimizations
- [Flo: a Semantic Foundation for Progressive Stream Processing](https://arxiv.org/abs/2411.08274) (POPL 2025) - Formal semantics

## Contributing

We welcome contributions to Hydro!
This is an experimental, research-driven project with opportunities for impactful contributions.

### Getting Started

1. **Fork and clone** the repository
2. **Install Rust** 1.90.0 or later (managed automatically via `rust-toolchain.toml`)
3. **Run tests**: `cargo test`
4. **Check formatting and lints**: `./precheck.bash`

### Development Guidelines

- Follow [Conventional Commits](https://www.conventionalcommits.org/) for PR titles
- Run `./precheck.bash` before submitting PRs
- Update snapshot tests with `cargo insta review`
- See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines

### Repository Structure

- **`hydro_lang/`** - High-level Hydro framework
- **`dfir_rs/`**, **`dfir_lang/`**, **`dfir_macro/`** - DFIR runtime and compiler
- **`hydro_deploy/`** - Multi-cloud deployment framework
- **`lattices/`** - Lattice data structures library
- **`hydro_std/`** - Standard library of distributed protocols
- **`docs/`** - Documentation website source

## License

Hydro is licensed under the **Apache License 2.0**.

See [LICENSE](LICENSE) for full license text.

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

See [CITATION.cff](CITATION.cff) for additional citation formats.

---

<p align="center">
Built with ❤️ by the <a href="https://hydro.run">Hydro Project</a>
</p>
