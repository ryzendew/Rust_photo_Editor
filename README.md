# Rust Photo

A powerful photo editing application written in Rust for Linux, inspired by professional image editors.

## Features

- Advanced layer system with adjustments, masks, and blend modes
- Non-destructive editing workflow
- Vector tools and text editing
- RAW image development
- Selection tools with advanced masking options
- Filters and effects with real-time preview
- Color management system
- GPU-accelerated processing

## Development Status

This project is currently under active development. See the roadmap section for upcoming features.

## Platform Support

**This project is currently Linux-only.**
Future support for other platforms is not planned at this time.

## Dependencies

### Required Libraries

The following libraries are required to build and run Rust Photo:

- Rust 1.70.0 or newer
- GTK4 4.6.0 or newer
- Cairo 1.16.0 or newer
- libadwaita 1.2.0 or newer
- image and imageproc libraries (handled by Cargo)
- dirs 5.0.0 or newer (for configuration handling)
- GPU support libraries depending on chosen backend (Vulkan, WGPU, etc.)

### Installing Dependencies

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install build-essential libgtk-4-dev libadwaita-1-dev libcairo2-dev \
                 libglib2.0-dev libpango1.0-dev libgdk-pixbuf2.0-dev \
                 librust-atk-dev libsoup-3.0-dev curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Fedora
```bash
sudo dnf install gtk4-devel libadwaita-devel cairo-devel pango-devel \
                 gdk-pixbuf2-devel atk-devel glib2-devel gcc \
                 libsoup3-devel curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Arch Linux
```bash
sudo pacman -S gtk4 libadwaita cairo pango gdk-pixbuf2 atk base-devel \
               glib2 libsoup3 curl
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### macOS
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install gtk4 libadwaita cairo pango gdk-pixbuf atk glib pkg-config
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

#### Windows
1. Install [Rust](https://www.rust-lang.org/tools/install)
2. Install [MSYS2](https://www.msys2.org/) and run from MSYS2 MinGW64 terminal:
```bash
pacman -S mingw-w64-x86_64-gtk4 mingw-w64-x86_64-libadwaita \
          mingw-w64-x86_64-cairo mingw-w64-x86_64-pango \
          mingw-w64-x86_64-gdk-pixbuf2 mingw-w64-x86_64-atk \
          mingw-w64-x86_64-toolchain base-devel
```
3. Add the MSYS2 bin directory to your PATH (typically `C:\msys64\mingw64\bin`)

## Building from Source

### Build Instructions

```bash
# Clone the repository
git clone https://github.com/yourusername/rust_photo.git
cd rust_photo

# Build in development mode
cargo build

# Run the application
cargo run
```

For production builds:

```bash
cargo build --release
```

## Performance Benchmarking

The project includes benchmark tests to measure performance of key components:

### Running Benchmarks

```bash
# Run all benchmarks
cargo bench

# Run specific benchmark group
cargo bench --bench filters_benchmark
cargo bench --bench layer_benchmark
cargo bench --bench selection_benchmark
cargo bench --bench raw_processing_benchmark
```

### Benchmark Categories

- **Filters Benchmark**: Tests performance of various image filters (blur, sharpen, etc.)
- **Layer Benchmark**: Measures layer composition and blend mode performance
- **Selection Benchmark**: Tests performance of selection operations
- **RAW Processing Benchmark**: Measures RAW image development performance

Benchmark results are generated in HTML format in the `target/criterion` directory.

## Configuration

User preferences are stored in `~/.config/rust_photo/`

## Troubleshooting

### Common Issues

1. **Missing GTK libraries**: Ensure you have installed all required GTK4 and libadwaita development packages.
2. **GPU acceleration issues**: Check that your system has compatible GPU drivers installed.
3. **Build errors related to Cairo**: Make sure you have the Cairo development packages installed.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 