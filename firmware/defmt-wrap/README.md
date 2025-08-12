# defmt-wrap

A wrapper around the [defmt](https://github.com/knurling-rs/defmt) crate that extends its functionality by enabling simultaneous logging to both the standard defmt output and a custom back channel.

## Features

- **Dual-channel logging**: Send log messages to both defmt's standard output and a custom back channel
- **No-std compatible**: Designed for resource-constrained embedded systems
- **Full defmt compatibility**: Reexports all defmt functionality
- **Simple API**: Drop-in replacement for defmt with minimal setup
- **Memory efficient**: Uses the same memory-efficient formatting as defmt

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
defmt = { version = "0.3.100", git = "https://github.com/yourusername/defmt-wrap" }
```

## Usage

1. **Set up the crate**

We want this crate to be called `defmt` in your crate.
In the Cargo.toml file add:

```
[dependencies]
defmt = { version = "0.3", features = ["alloc"] }

[patch.crates-io]
defmt = { path = '../defmt-wrap' }
```

2. **Import the crate**

```rust
use defmt;
```

3. **Set up a back channel callback function**

```rust
fn my_callback(message: String) {
    // Process the log message as needed
    // For example, send it over UART, BLE, or store in memory
}

fn main() {
    // Register the callback function
    defmt::back_channel::set_callback(my_callback);

    // Now use defmt logging macros as usual
    defmt::info!("System initialized");
    defmt::warn!("Low memory: only {} bytes remaining", available_memory);
    defmt::error!("Failed to initialize device: {}", error_code);
}
```

## API Reference

This crate reexports all functionality from defmt while enhancing the following macros:

- `info!`: Logs an info message to both defmt and the back channel
- `warn!`: Logs a warning message to both defmt and the back channel
- `error!`: Logs an error message to both defmt and the back channel

Each back channel message is prefixed with a level indicator:
- "I: " for info messages
- "W: " for warning messages
- "E: " for error messages

### Back Channel Module

The `back_channel` module provides functions for setting up and managing the custom logging channel:

- `set_callback(callback: fn(String))`: Register a callback function to receive log messages
- `get_callback() -> Option<fn(String)>`: Get the currently registered callback function

## Example

```rust
use defmt;
use core::sync::atomic::{AtomicUsize, Ordering};

// Example memory buffer to store log messages
static mut LOG_BUFFER: [u8; 1024] = [0; 1024];
static LOG_POSITION: AtomicUsize = AtomicUsize::new(0);

fn store_log(message: String) {
    let bytes = message.as_bytes();
    let current_pos = LOG_POSITION.load(Ordering::Relaxed);

    // Ensure we don't overflow the buffer
    if current_pos + bytes.len() < LOG_BUFFER.len() {
        unsafe {
            LOG_BUFFER[current_pos..current_pos + bytes.len()].copy_from_slice(bytes);
        }
        LOG_POSITION.store(current_pos + bytes.len(), Ordering::Relaxed);
    }
}

fn main() {
    // Register our callback
    defmt::back_channel::set_callback(store_log);

    // These messages will be sent to both defmt output and our log buffer
    defmt::info!("Starting application");
    defmt::warn!("Resource {} is running low", "memory");
    defmt::error!("Critical error: {}", 42);
}
```

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Contributing

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you shall be dual licensed as above, without any additional terms or conditions.
