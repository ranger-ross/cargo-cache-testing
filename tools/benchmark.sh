#!/usr/bin/env bash
set -euo pipefail

phase=${1:?expected fetch or build}
strategy=${BENCHMARK_STRATEGY:?BENCHMARK_STRATEGY is required}
command=(cargo)
cache_args=()

case "$strategy" in
  no-cache|rust-cache|kache) ;;
  boxington)
    if [[ "$phase" == build ]]; then
      command=(mbx)
    fi
    ;;
  native-rust-cache|native-build-buddy)
    command=("${NATIVE_CARGO:?NATIVE_CARGO is required}")
    export RUSTUP_AUTO_INSTALL=0
    cache_args=(-Zshared-blob-storage)
    if [[ "$strategy" == native-build-buddy && "$phase" == build ]]; then
      : "${BUILD_BUDDY_API_KEY:?BUILD_BUDDY_API_KEY is required}"
      cache_args+=(
        --config 'cache.remote.url="grpcs://remote.buildbuddy.io"'
        --config "cache.remote.instance-name=\"${CACHE_SCOPE:?CACHE_SCOPE is required}\""
        --config 'cache.remote.api-key-env="BUILD_BUDDY_API_KEY"'
      )
    fi
    ;;
  *) echo "Unknown benchmark strategy: $strategy" >&2; exit 1 ;;
esac

start=$SECONDS
case "$phase" in
  fetch)
    "${command[@]}" fetch --locked
    metric=FETCH_SECS
    ;;
  build)
    read -r -a build_args <<< "${BUILD_ARGS:---release --locked}"
    "${command[@]}" build "${build_args[@]}" --timings "${cache_args[@]}"
    metric=BUILD_SECS
    ;;
  *) echo "Unknown benchmark phase: $phase" >&2; exit 1 ;;
esac
elapsed=$((SECONDS - start))
printf '%s %s took %ss\n' "$strategy" "$phase" "$elapsed"
if [[ -n "${GITHUB_ENV:-}" ]]; then
  printf '%s=%s\n' "$metric" "$elapsed" >> "$GITHUB_ENV"
fi
