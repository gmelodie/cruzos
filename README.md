# cruzOS
My own OS written in Rust

```bash
sudo apt install qemu-kvm
rustup toolchain install nightly
rustup override set nightly
cargo install bootimage
rustup component add llvm-tools-preview
```

## Memory allocators
- **Bump Allocator:** grows in the same direction once all allocated blocks are deallocated, reset memory.
- **Linked List Allocator:** freed blocks with at least 16 bytes are put in a linked list. Reutilizes suitable blocks. (obs: if you deallocate 8 bytes, those are lots until all references are deallocated and memory is reset like in bump allocator)

## Async
There is a naïve task scheduler at `/src/task/simple_executor.rs`. Tasks can be spawned after executor starts `run`ing. Tasks are not processes. They don't have their own contexes or memory.

## Processes
Initial idea of processes including context switching and preemptive multitasking achieved due to [bendudson's awesome tutorial](https://github.com/bendudson/EuraliOS/blob/main/doc/journal/01-interrupts-processes.org).


## TODOs
- VGA printing queue with daemon job. If can't lock vga, add it to queue.
- Fix page fault when keyboard interrupt happens
- Syscalls
    - pkill
    - ps
    - shutdown
