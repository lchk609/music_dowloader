# Music Downloader

A feature-rich, cross-platform music downloader built in Rust with a sleek GUI. Download individual songs and entire playlists from various online sources with ease.

## Features

- 🎵 **Download Individual Songs** - Download single tracks with metadata preservation
- 📋 **Playlist Downloads** - Batch download entire playlists at once
- 🎨 **Modern GUI** - Intuitive Slint-based user interface
- ⚙️ **Configurable Codecs** - Support for multiple audio formats
- 🔄 **Async Processing** - Fast, non-blocking downloads using Tokio
- 🛠️ **Multi-threaded** - Efficient resource utilization with threadpool support
- 💾 **Smart Directory Handling** - Automatic platform-specific directory management
- 🔍 **Filename Sanitization** - Safe, valid filenames across all platforms

## Prerequisites

- **Rust** 1.70+ (2024 edition)
- **yt-dlp** - Required for downloading content (automatically integrated via the yt-dlp crate)

## Installation

### From Source

```bash
git clone https://github.com/yourusername/music_downloader.git
cd music_downloader
cargo build --release
```

The compiled binary will be available at `target/release/music_downloader`.

## Usage

### Running the Application

```bash
cargo run --release
```

Or if you've already built it:

```bash
./target/release/music_downloader
```

### Using the GUI

1. **Start the application** - A window will open with the music downloader interface
2. **Enter URL** - Paste a URL to a song or playlist
3. **Configure Settings** - Select your preferred audio codec and output directory
4. **Start Download** - Click the download button to begin
5. **Monitor Progress** - Track the download status in real-time

## Configuration

The application stores configuration in platform-specific directories:

- **Windows**: `%APPDATA%\music_downloader\`
- **macOS**: `~/Library/Application Support/music_downloader/`
- **Linux**: `~/.config/music_downloader/`

Configuration is stored in JSON format for easy customization.

## Project Structure

```
src/
├── main.rs              # Application entry point
├── config/              # Configuration management
├── dowloaders/          # Download engine implementations
│   ├── music.rs         # Single song downloader
│   ├── playlist.rs      # Playlist downloader
│   └── dowloader_base.rs# Base downloader interface
├── ui/                  # GUI components (Slint)
│   └── components/      # Reusable UI elements
├── events/              # Event handling system
├── enums/               # Type definitions
│   └── codec.rs         # Audio codec options
└── setup.rs             # Initialization logic
```

## Build Features

### Optional Features

- **tracing** - Enable debug tracing for development (useful with `RUST_LOG=debug`)

To enable:

```bash
cargo run --features tracing
```

## Development

### Building for Development

```bash
cargo build
```

### Running with Tracing

For detailed debug output:

```bash
RUST_LOG=debug cargo run --features tracing
```

### Building for Release

Optimized build for production:

```bash
cargo build --release
```

## Dependencies

Key dependencies:

- **slint** - Cross-platform GUI framework
- **tokio** - Async runtime and concurrency
- **yt-dlp** - Audio/video download library
- **serde/serde_json** - Serialization/deserialization
- **chrono** - Date and time handling
- **uuid** - Unique identifier generation
- **directories** - Platform-specific directory handling

## Supported Platforms

- ✅ Linux
- ✅ macOS
- ✅ Windows

## License

[Add your license here]

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Troubleshooting

### Downloads not working

- Ensure `yt-dlp` is installed and accessible in your system PATH
- Check that you have an active internet connection
- Verify the URL is valid and supported by yt-dlp

### GUI not appearing

- On Linux, ensure you have required graphics libraries installed
- Check terminal output for error messages

### File permission errors

- Ensure you have write permissions to the output directory
- Try running with elevated permissions (if necessary for your platform)

## Support

For issues, feature requests, or questions, please open an issue on GitHub.
