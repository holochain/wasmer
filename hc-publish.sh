#!/usr/bin/env sh

cargo publish --manifest-path lib/types/Cargo.toml && \
  cargo publish --manifest-path lib/vm/Cargo.toml && \
  cargo publish --manifest-path lib/object/Cargo.toml && \
  cargo publish --manifest-path lib/derive/Cargo.toml && \
  cargo publish --manifest-path lib/compiler/Cargo.toml && \
  cargo publish --manifest-path lib/compiler-cranelift/Cargo.toml && \
  cargo publish --manifest-path lib/compiler-llvm/Cargo.toml --no-verify && \
  cargo publish --manifest-path lib/compiler-singlepass/Cargo.toml && \
  cargo publish --manifest-path lib/api/Cargo.toml && \
  cargo publish --manifest-path lib/middlewares/Cargo.toml
