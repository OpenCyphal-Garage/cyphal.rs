# Example

Shows usage on a embedded device.

In this case it`s a STM32G431 board.

## Required Tools

- [flip-link](https://github.com/knurling-rs/flip-link)
  
    ```
    cargo install flip-link
    ```

- [probe-run](https://github.com/knurling-rs/probe-run)

    ```
    cargo install probe-run
    ```

## How to run

```bash
cargo run --release
```

## How to debug

1. Install recommended VsCode Extension [cortex-debug](https://github.com/Marus/cortex-debug)
2. Open OpenOCD server (this config is a custom one)
   
   ```bash
    openocd -f "board/stm32g431b.cfg"
   ```

   config:
    ```
    source [find interface/stlink-dap.cfg]
    transport select dapdirect_swd
    source [find target/stm32g4x.cfg]
    ```
3. Start Debug session in VsCode [Debug (OpenOCD)]