# Test Frameworks Removed - Migrated to cuddly-giggle

This document tracks the test frameworks that have been migrated from this repository to the cuddly-giggle repository.

## Migration Date
Completed on: [Current Date]

## Removed Packages

The following packages have been removed from this repository and are now maintained in the cuddly-giggle repository:

### 1. hydro_test
- **Original Location**: `/projects/sandbox/hydro/hydro_test/`
- **New Location**: `/projects/sandbox/cuddly-giggle/test_frameworks/hydro_test/`
- **Purpose**: Main test framework for distributed, cluster, and local testing
- **Repository**: https://github.com/kyleandmikescrazyadventure/cuddly-giggle

### 2. example_test
- **Original Location**: `/projects/sandbox/hydro/example_test/`
- **New Location**: `/projects/sandbox/cuddly-giggle/test_frameworks/example_test/`
- **Purpose**: Test utility for cargo examples
- **Repository**: https://github.com/kyleandmikescrazyadventure/cuddly-giggle

### 3. multiplatform_test
- **Original Location**: `/projects/sandbox/hydro/multiplatform_test/`
- **New Location**: `/projects/sandbox/cuddly-giggle/test_frameworks/multiplatform_test/`
- **Purpose**: Multiplatform test attribute macro
- **Repository**: https://github.com/kyleandmikescrazyadventure/cuddly-giggle

### 4. include_mdtests
- **Original Location**: `/projects/sandbox/hydro/include_mdtests/`
- **New Location**: `/projects/sandbox/cuddly-giggle/test_frameworks/include_mdtests/`
- **Purpose**: Markdown doctest inclusion macro
- **Repository**: https://github.com/kyleandmikescrazyadventure/cuddly-giggle

## Changes Made to Hydro Repository

### Workspace Configuration
Updated `Cargo.toml` workspace members to exclude migrated packages:
- Removed `"example_test"`
- Removed `"hydro_test"`
- Removed `"include_mdtests"`
- Removed `"multiplatform_test"`

### Dependency Updates
Updated `dfir_rs/Cargo.toml` to point to new locations in cuddly-giggle:
- `example_test` now references `../../cuddly-giggle/test_frameworks/example_test`
- `include_mdtests` now references `../../cuddly-giggle/test_frameworks/include_mdtests`
- `multiplatform_test` now references `../../cuddly-giggle/test_frameworks/multiplatform_test`

### Directories Removed
- `/projects/sandbox/hydro/hydro_test/` (removed)
- `/projects/sandbox/hydro/example_test/` (removed)
- `/projects/sandbox/hydro/multiplatform_test/` (removed)
- `/projects/sandbox/hydro/include_mdtests/` (removed)

## Using the Migrated Test Frameworks

### For Development
If you need to use these test frameworks during development, they are available in the cuddly-giggle repository:

```bash
# Clone cuddly-giggle if not already present
cd /projects/sandbox
git clone https://github.com/kyleandmikescrazyadventure/cuddly-giggle.git

# Reference them in your Cargo.toml
[dev-dependencies]
example_test = { path = "../../cuddly-giggle/test_frameworks/example_test" }
multiplatform_test = { path = "../../cuddly-giggle/test_frameworks/multiplatform_test" }
include_mdtests = { path = "../../cuddly-giggle/test_frameworks/include_mdtests" }
```

### For Production Use
These packages may be published to crates.io. Check the cuddly-giggle repository for published versions:

```toml
[dev-dependencies]
example_test = "0.0.0"
multiplatform_test = "0.5.0"
include_mdtests = "0.0.0"
```

## Test Files
Test files for individual packages (dfir_rs, lattices, hydro_lang, etc.) remain in their respective packages within this repository. Only the test framework packages and their associated utilities have been migrated.

## Retained in Hydro
The following remain in the hydro repository:
- `hydro_build_utils` - Still used by multiple packages in hydro
- All package-specific test directories (`dfir_rs/tests/`, `lattices/tests/`, etc.)
- CI/CD configurations (though copies were made to cuddly-giggle)
- GitHub Actions used by hydro CI

## Documentation Updates Needed

The following documentation may need updates to reflect the migration:
1. **README.md** - Remove references to migrated test frameworks
2. **CONTRIBUTING.md** - Update testing instructions
3. **RELEASING.md** - Update release process
4. **API documentation** - Update links to test framework docs

## CI/CD Notes

The hydro CI pipeline continues to work with the test files that remain in each package. The test frameworks in cuddly-giggle can be referenced if needed via path dependencies (as demonstrated in `dfir_rs/Cargo.toml`).

## Rollback Instructions

If you need to rollback this migration:

```bash
# The original packages are preserved in Git history
cd /projects/sandbox/hydro

# Restore from previous commit
git checkout <previous-commit-hash> -- hydro_test example_test multiplatform_test include_mdtests

# Restore workspace members in Cargo.toml
git checkout <previous-commit-hash> -- Cargo.toml

# Restore dfir_rs dependencies
git checkout <previous-commit-hash> -- dfir_rs/Cargo.toml
```

## More Information

For detailed information about the migration, see the cuddly-giggle repository:
- [TEST_FRAMEWORKS_MIGRATION.md](https://github.com/kyleandmikescrazyadventure/cuddly-giggle/blob/main/TEST_FRAMEWORKS_MIGRATION.md)
- [test_frameworks/README.md](https://github.com/kyleandmikescrazyadventure/cuddly-giggle/blob/main/test_frameworks/README.md)

## Contact

For questions about:
- **Hydro packages**: See hydro repository maintainers
- **Test frameworks**: See cuddly-giggle repository maintainers
- **Migration issues**: Contact both teams

---

This migration consolidates test framework development in a dedicated repository while maintaining compatibility with hydro packages.
