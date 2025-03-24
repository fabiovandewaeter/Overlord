# YetAnotherMinecraftClone

## Commands
[install dependencies](https://bevyengine.org/learn/quick-start/getting-started/setup/#installing-os-dependencies)


### [Alternative Linkers](https://bevyengine.org/learn/quick-start/getting-started/setup/#alternative-linkers) :
- Ubuntu: `sudo apt-get install lld clang`
- Fedora: `sudo dnf install lld clang`
- Arch: `sudo pacman -S lld clang`
- Windows: Ensure you have the latest cargo-binutils as this lets commands like cargo run use the LLD linker automatically :
```shell
    cargo install -f cargo-binutils
    rustup component add llvm-tools-preview
```
- MacOS: On MacOS, the default system linker ld-prime is faster than LLD.

### Dev
#### Optimisations
- Cranelift :
```shell
rustup component add rustc-codegen-cranelift-preview --toolchain nightly
```

- compilation when dev :
```shell
cargo run --features bevy/dynamic_linking
```

### Test

```shell
cargo test
```

### Release

```shell
cargo run --release
```

