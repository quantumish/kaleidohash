# kaleidohash 

## About 
Simple-ish implementation of a rainbow table designed to crack SHA-1 passwords. For a detailed explanation, see the writeup [here](https://quantumish.github.io/kaleidohash.html).

## Usage

### Building
<details>
<summary>Install Rust and `cargo`</summary>
<br>
From the [Rust installation guide](https://doc.rust-lang.org/cargo/getting-started/installation.html):
```bash
curl https://sh.rustup.rs -sSf | sh
```
</details>

### Running
You can run the code with:
```bash
cargo run --release --bin kaleidohash
```

You can experiment with the code by modifying the `main()` function by tweaking the arguments to `RainbowTable::new()` - `main()` will then generate 100 random hashes and print any successful password cracks.
