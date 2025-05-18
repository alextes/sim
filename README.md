# sim

![sim logo](sim.png)

## Building from Source

For Debian/Ubuntu-based systems, you\'ll need to install the following development libraries:

```bash
sudo apt-get update
sudo apt-get install -y libsdl2-dev libsdl2-image-dev
```

For macOS, you can use Homebrew to install the necessary libraries:

```bash
brew install sdl2 sdl2_image
```

You also need Rust and Cargo installed, preferably via `rustup`:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
# Follow the on-screen instructions, then ensure the stable toolchain is default:
rustup default stable
```
