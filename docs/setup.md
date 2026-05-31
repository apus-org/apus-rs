# Development Setup

How to set up a machine to build, flash, and debug apus-rs on the APUS board
(STM32F072VBT6, Arm Cortex-M0, target `thumbv6m-none-eabi`).

## 1. Rust toolchain

[Install Rust](https://rust-lang.org/tools/install/), then add the bare-metal target:

```bash
rustup target add thumbv6m-none-eabi
```

Binary-inspection tools (used for size reports and `objcopy`):

```bash
rustup component add llvm-tools
cargo install cargo-binutils
```

Verify the toolchain and target:

```bash
rustup show
```

## 2. Flashing with probe-rs

[Install probe-rs](https://probe.rs/docs/getting-started/installation/). The build target,
runner, and linker flags are preconfigured in [.cargo/config.toml](../.cargo/config.toml),
so flashing and running an example is just:

```bash
cargo run --example board
```

## 3. Linker script

The MCU memory layout lives in [memory.x](../memory.x) (128 KB flash, 16 KB RAM), matched to
the STM32F072VB [datasheet](https://www.st.com/resource/en/datasheet/stm32f072vb.pdf).

## 4. Editor / debugging (optional)

- VS Code debug configuration: [.vscode/launch.json](../.vscode/launch.json)
- SVD file for the peripheral register view: [device/STM32F0x2.svd](../device/STM32F0x2.svd)

## Hardware reference

- MCU: [STM32F072VB](https://www.st.com/en/microcontrollers-microprocessors/stm32f072vb.html) — Arm Cortex-M0, Armv6-M
- Rust target: [thumbv6m-none-eabi](https://doc.rust-lang.org/rustc/platform-support/thumbv6m-none-eabi.html)