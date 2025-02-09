# A Minimal Rust OS Kernel

Based on examples provided in https://github.com/rust-osdev/bootloader

To compile, ensure that your linux/wsl2 environment target is ready and you have qemu installed:
```
sudo apt install qemu-system
```

The main project is configured with build-dependencies to build bootimage & run qemu when invoking `cargo build` or `cargo run`.

The actual kernel is implemented in `kernel/src`.
