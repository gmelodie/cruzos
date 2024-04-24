# cruzOS
My own OS written in Rust

```bash
sudo apt install qemu-kvm
rustup toolchain install nightly
rustup override set nightly
cargo install bootimage
rustup component add llvm-tools-preview
```

## TODOs
- DMA over PIO (use this to implement keyboard buffer)
- UEFI over BIOS
- USB
- HDMI or DisplayPort over VGA
- Process scheduling (requirements??)
