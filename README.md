# Setup for Embedded Rust on APUS Board

1. [Install Rust](https://rust-lang.org/tools/install/)

2. Install Rust Tools for Binaries 
```bash
    rustup component add llvm-tools
```

```bash
    cargo install cargo-binutils
```

3. Install Hardware Support

- Look up Hardware-Information:
    - MCU: [STM32F072VB](https://www.st.com/en/microcontrollers-microprocessors/stm32f072vb.html)
        - Family: [ARM Cortex-M0](https://developer.arm.com/Processors/Cortex-M0)
            - Architecture: Armv6-M
            - ISA: Thumb / Thumb-2 subset

- Search for Hardware support in rust documentation
    - Platform: [Bare Armv6-M](https://doc.rust-lang.org/beta/rustc/platform-support.html)
        - Target: [thumbv6m](https://doc.rust-lang.org/beta/rustc/platform-support/thumbv6m-none-eabi.html)

- Add Target Support: 
```bash
rustup target add thumbv6m-none-eabi
```

- Verify Installation:
```bash
rustup show
```

- Create Memory File (Linker Script) for MCU according to Memory Map of Datasheet:
    - [Datasheet](https://www.st.com/resource/en/datasheet/stm32f072vb.pdf)
        - [Linker Script](./memory.x)


# Building/Debugging
- [Install Probe-rs](https://probe.rs/docs/getting-started/installation/)

- Add information for build in [config](./.cargo/config.toml)
    - Target
    - Runner
    - Linker-Flags

- [Create launch.json file for debugging in vscode](./.vscode/launch.json)
- [Provide SVD-File of MCU for advanced debugging (periphal view...)](./device/STM32F0x2.svd)
    - [stm32f072vb-cad-resources](stm32f072vb.html#cad-resources)


# Packages
- Install dependencies listed in [Cargo.toml](./Cargo.toml)
```bash
cargo install
```
