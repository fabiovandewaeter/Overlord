# YetAnotherMinecraftClone

## Commands
[install dependencies](https://bevyengine.org/learn/quick-start/getting-started/setup/#installing-os-dependencies)

[Alternative Linkers](https://bevyengine.org/learn/quick-start/getting-started/setup/#alternative-linkers)



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

