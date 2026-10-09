# Cargo cache benchmarks

The workflows compare no cache, Swatinem/rust-cache, kache, mr-boxington,
native Cargo shared storage with actions/cache, and native Cargo shared storage
with BuildBuddy. sccache is excluded for now.

- `cache-benchmark.yml` builds this repository and runs its command-line program.
- `cache-benchmark-zed.yml` builds the Zed editor on Ubuntu.
- `benchmark-project.yml` contains the reusable benchmark jobs.
- `tools/benchmark.sh` runs the timed dependency fetch and build steps.

Run either project workflow from the Actions tab. Both also run on pushes to
`main`. Zed's manual workflow accepts a branch, tag, or commit. Pin a commit when
comparing repeated warm-cache runs, since upstream `main` can change.

The reusable workflow resolves the project ref once and uses that commit for
all strategies. It reads the project's `rust-toolchain.toml` or `rust-toolchain`
file, falling back to stable. Every strategy uses the same selected toolchain,
build arguments, and project setup. The native strategies use the custom Cargo
executable committed in `tools/cargo/cargo`.

## Zed

The Zed caller selects `--package zed --bin zed` in release mode. It follows
Zed's [Linux dependency setup](https://github.com/zed-industries/zed/blob/main/docs/src/development/linux.md)
with `script/linux` and downloads its WASI SDK with `script/download-wasi-sdk`.
The smoke command uses `zed --help`, which exits before GUI startup.

Each strategy has a 180-minute timeout. Setup frees unused Android and .NET
SDKs on the disposable Ubuntu runner. Zed is a large build, so future revisions
may still require a runner with more disk or memory. Set `runner` to a suitable
Linux x86-64 runner if the standard hosted runner cannot accommodate it.

## Add a project

Copy a project caller into `.github/workflows/cache-benchmark-<project>.yml`.
A project without system dependencies only needs:

```yaml
name: cache-benchmark-example

on:
  push:
    branches: [main]
  workflow_dispatch:

permissions:
  contents: read

jobs:
  benchmark:
    uses: ./.github/workflows/benchmark-project.yml
    with:
      project: example
      repository: owner/repository
      ref: main
      build-args: --release --locked --package example
    secrets: inherit
```

Use a unique lowercase project slug with letters, numbers, and hyphens. The
repository must be publicly readable and have a Cargo workspace at its root.
Build arguments are a single line of whitespace-separated Cargo arguments,
without shell quotes or expansions. The benchmark adds `--timings` itself.

Optional inputs:

| Input | Default | Purpose |
|---|---|---|
| `ref` | Repository default branch | Branch, tag, or commit to build |
| `toolchain` | Project toolchain file, otherwise stable | Override the Rust toolchain |
| `build-args` | `--release --locked` | Select packages, binaries, features, or a Cargo profile |
| `setup-command` | Empty | Install system dependencies or set build environment variables |
| `smoke-command` | Empty | Exercise the built program after a successful build |
| `runner` | `ubuntu-24.04` | Select a Linux x86-64 runner compatible with the native Cargo executable |
| `timeout-minutes` | `60` | Limit each strategy job |

Setup and smoke commands run as Bash scripts in the checked-out project root.
Setup runs before cache restoration and before the selected toolchain is
installed. Export settings for subsequent steps by appending to `GITHUB_ENV`:

```yaml
      setup-command: |
        sudo apt-get update
        sudo apt-get install -y libexample-dev
        echo "EXAMPLE_SETTING=value" >> "$GITHUB_ENV"
      smoke-command: ./target/release/example --help
```

These scripts and the project build run with the caller's privileges. Use
trusted repositories and refs, especially when passing secrets. Keep Cargo's
output in the default `target` directory so cache actions and timing artifact
uploads can find it.

## Caches and results

Caches are separated by project, repository, toolchain selection, build
arguments, and setup script. The native BuildBuddy backend uses the same scope
as its remote instance name. Changing the reusable scope generation starts a
fresh cache namespace. Existing caches from the old single-project workflow
are not reused after this cutover.

Configure the repository secret `BUILD_BUDDY_API_KEY` to include the BuildBuddy
strategy. Without it, the workflow reports a notice and runs the other five
strategies. Public project checkout does not require an additional token.

Each job reports fetch and build seconds and uploads a
`cargo-timings-<project>-<strategy>` artifact. The resolver summary records the
project commit and toolchain. Native jobs also report local shared-storage
size and file count. Local shared-storage size is not BuildBuddy remote size.
Cache actions print their archive sizes and available cache statistics in the
job logs.

Project setup, tool installation, archive restoration, artifact upload, smoke
commands, and cache saving are outside the timed build. Remote cache work done
by the build process is inside it. Compare complete job duration as well as
build duration. A first run populates caches, and later runs measure reuse.
Manual runs save mr-boxington's cache so a manually added project can warm it.
