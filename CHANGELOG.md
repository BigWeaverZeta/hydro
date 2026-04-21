# Changelog

## 2026-04-21

### Changed (Kiro)

- Moved `benches` crate (microbenchmarks) from this repository to the [hydro-deps](https://github.com/BigWeaverZeta/hydro-deps) repository.
- Removed `benches` from workspace members in `Cargo.toml`.
- Removed `.github/workflows/benchmark.yml` CI workflow (benchmarks now run from hydro-deps).
- Updated `CONTRIBUTING.md` to reference the new benchmark location.
