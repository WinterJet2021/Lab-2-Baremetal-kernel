# A Minimal Rust OS Kernel

Based on examples provided in https://github.com/rust-osdev/bootloader

To compile, ensure that your linux/wsl2 environment target is ready:
```
sudo apt install qemu-system
sudo apt install ovmf
rustup target add x86_64-unknown-none
rustup component add llvm-tools-preview
rustup override set nightly
```

The main project is configured with build-dependencies to build bootimage & run qemu when invoking `cargo build` or `cargo run`.

The actual kernel is implemented in `kernel/src`.
