# Goosling

A simple x86_64 kernel

[![Build](https://github.com/goos-org/goosling/actions/workflows/workflow.yml/badge.svg?branch=main)](https://github.com/goos-org/goosling/actions/workflows/workflow.yml)

## Plans
- [x] Boot using limine
- [x] Memory management
- [ ] Multitasking
- [ ] Interrupts/syscalls
- [ ] Console input/output
- [ ] Simple driver system

## Building
The kernel can be built using `cargo build`.
You can also create a bootable iso using the `make iso` (`all`/`default`) target.
This will automatically download limine and build the kernel.
A release build can be made by passing the `RELEASE=1` environment variable, or using `make release`.

## Usage
You can run the iso natively by burning it to a dvd or writing it to a usb drive (with [rufus](https://rufus.ie/en/), for example).
You can also use `qemu-system-x86_64` to run the iso (`qemu-system-x86_64 -cdrom build/goosling.iso`).

## Debugging
In order to debug with qemu, you can simply pass the `-S` and `-s` flags.
This will create a gdb remote on `tcp:1234`.
You can connect with gdb (`target remote localhost:1234`) or with your IDE:
- CLion:
  - Add a "Remote Debug" run configuration
  - Set `'target remote' args` to `localhost:1234`
  - Set `symbol file` to `build/kernel`
  - Run qemu with `-S` and `-s` first, then debug
- Visual Studio Code:
  - Add this to your `launch.json`: 
    ```json
    {
        "type": "gdb",
        "request": "attach",
        "name": "Attach to gdbserver",
        "executable": "build/kernel",
        "target": "localhost:1234",
        "remote": true,
        "cwd": "${workspaceRoot}",
        "gdbpath": "path/to/your/gdb"
    }
    ```
