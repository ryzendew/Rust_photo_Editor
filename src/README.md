# Rust Photo 0.1

A feature-rich image editor built in Rust with GTK4, inspired by Affinity Photo. This project aims to create a professional-grade image editor with GPU acceleration.

## Features

- **Vector-First Approach**: Clean lines with infinite detail and scaling capabilities
- **Modern UI**: Clean, dark interface with customizable panels and tools
- **Layer-based editing**: Non-destructive editing with support for blending modes and layer masks
- **Advanced selection tools**: Rectangle, ellipse, freehand, and smart selections
- **Powerful filters**: Blur, sharpen, distort and more with real-time previews
- **GPU Acceleration**: CUDA (NVIDIA), ROCm (AMD), and OpenCL support for fast processing
- **Color management**: Support for various color spaces and bit depths

## Vector-First Design

Unlike traditional image editors that work primarily with raster/bitmap graphics, Affinity Photo RS takes a vector-first approach:

- All drawing operations create vector paths by default
- Lines, shapes, and brush strokes maintain perfect quality at any zoom level
- All operations remain editable and non-destructive
- Vector paths are only rasterized when absolutely necessary 
- Export in both vector (SVG, PDF) and raster (PNG, JPEG) formats

This approach provides several advantages:
- Infinitely scalable artwork without quality loss
- Smaller file sizes for complex designs
- Precise editing of paths and shapes after creation
- Better quality output for print and high-resolution displays

## System Requirements

- Rust 1.70 or newer
- GTK 4.8 or newer
- For GPU acceleration:
  - NVIDIA: CUDA toolkit 11.0+
  - AMD: ROCm 5.0+
  - Generic: OpenCL 1.2+

## Building from Source

1. Install dependencies:

```bash
# Ubuntu/Debian
sudo apt install build-essential libgtk-4-dev libadwaita-1-dev 

# Fedora
sudo dnf install gtk4-devel libadwaita-devel

# Arch Linux
sudo pacman -S gtk4 libadwaita
```

2. Clone the repository:

```bash
git clone https://github.com/yourusername/affinity_photo_rs.git
cd affinity_photo_rs
```

3. Build and run:

```bash
cargo build --release
cargo run --release
```

## GPU Acceleration

The application will automatically detect and use available GPU acceleration:

- NVIDIA GPUs: Enable with `--features gpu-cuda`
- AMD GPUs: Enable with `--features gpu-rocm`
- Other GPUs: Falls back to OpenCL

## Architecture

- **Vector**: Core vector graphics engine with infinite detail
- **Canvas**: Drawing area with event handling for vector creation
- **Tools**: Various vector and bitmap manipulation tools
- **Layers**: Non-destructive layer-based editing
- **Filters**: GPU-accelerated effects with vector-aware processing
- **UI**: GTK4 interface with dockable panels

## Current Status

This project is under active development and is not yet ready for production use.

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## License

This project is licensed under the MIT License - see the LICENSE file for details. 