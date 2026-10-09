# Cargo cache benchmarks

Compares no cache, Swatinem/rust-cache, sccache, kache, mr-boxington,
and native Cargo caching through actions/cache or BuildBuddy.

`cache-benchmark.yml` runs on pushes to `main` or manually from the Actions tab.
Jobs report fetch/build times, cache statistics, and Cargo timing artifacts.
Run again to compare warm caches. Set `BUILD_BUDDY_API_KEY` to include BuildBuddy.

## Add a project

Copy `.github/workflows/cache-benchmark.yml`, give it a unique workflow name,
and change the reusable workflow inputs:

```yaml
jobs:
  benchmark:
    uses: ./.github/workflows/benchmark-project.yml
    with:
      project: my-project
      repository: owner/my-project
      ref: main
      build-args: --release --locked --package my-project
    secrets: inherit
```

Use a unique lowercase project slug. All strategies build the same resolved
commit, with project-specific caches. Toolchains come from the project's
toolchain file, falling back to stable. Optional inputs include `toolchain`,
`setup-command`, `smoke-command`, `runner`, and `timeout-minutes`.
Use trusted repositories and setup scripts. Build arguments are whitespace-separated
without shell quoting. Keep Cargo output in the default `target` directory.
