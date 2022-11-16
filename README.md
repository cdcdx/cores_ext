# cpu cores extend

## Quote
```toml
[dependencies]
cores_ext = { git = "https://github.com/cdcdx/cores_ext", branch = "master" }
```

## Usage
```rust
    // Bind the CPU core of the current process
    let cpu = cores_ext::CPU::new();
    let bind_start = cpu.execution_units - cpu.physical_cores*cpu.sockets;
    let bind_end = cpu.execution_units;
    cores_ext::bind_cpu_ids(bind_start,bind_end);

    // Get the CPU core of the current process
    cores_ext::get_cpu_ids("test");
```

