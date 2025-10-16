
# stm32wba55-crypto

Minimal Rust + nRF54L15 example project.

## Setup

1. **Install JLink software and documentation**  
   ```bash
   yay -S jlink-software-and-documentation
   ```
2. **Install nrf-udev**
   ```bash
   yay -S nrf-udev
   ```
3. **Connect the nRF54L15 and run the example**
    ```bash
    cd app-core
    cargo run --bin blink
    ```
