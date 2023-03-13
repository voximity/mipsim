# mipsim

mipsim (*mip - sim*) is a lightweight MIPS assembly editor and simulator.

View, edit, assemble, execute, and step through MIPS assembly.

![image](https://user-images.githubusercontent.com/11506439/224566792-0acbd91d-bddd-41c9-870b-8e2d00e20ac1.png)

## Features

*Unchecked features are not implemented.*

- [x] Edit MIPS assembly
  - [x] Syntax highlighting
  - [x] Hover instructions for usage
  - [x] Open/save assembly files
  - [ ] Place breakpoints
- [x] Assemble MIPS
  - [x] Parse and load MIPS binary into processor
  - [x] Pseudo instructions
  - [ ] View assembled representation
- [x] Execute
  - [x] Step through MIPS instructions one instruction at a time
  - [x] Stdout syscalls
  - [x] Jumps and branches
  - [x] Stdin syscalls
  - [ ] Simulate a cycle rate and step at a certain frequency
  - [ ] Simulate the entire program instantaneously
  - [ ] Dynamic memory allocation syscall
- [ ] Analyze memory
  - [x] View register state
  - [ ] Explore program memory space in a hex dump
  - [ ] Modify program memory space

## Usage

Install [Rust](https://rust-lang.org), then

```
$ cargo build --release
```
