# FFmpeg Rust Video Player

A professional audio-video player built in Rust using FFmpeg and Slint UI. Features a modern interface with full playback controls, audio support, and cross-platform compatibility.

<img width="1006" height="740" alt="Screenshot from 2025-10-04 16-33-56" src="https://github.com/user-attachments/assets/7994dc7c-61d8-4429-9a73-8947359d8e21" />

## Features

### Video Playback
- **Multiple Input Sources**: Play videos from URLs or local files
- **Format Support**: MP4, MKV, AVI, MOV, WebM, FLV, WMV, M4V
- **High-Quality Rendering**: Hardware-accelerated video decoding with FFmpeg
- **Aspect Ratio Preservation**: Videos display with correct proportions

### Audio Playback
- **Full Audio Support**: Synchronized audio-video playback
- **Volume Control**: Real-time volume adjustment with slider
- **Multi-format Audio**: Supports various audio codecs
- **Audio-Video Sync**: Proper synchronization between audio and video

### UI
- **Responsive Layout**: Adapts to different window sizes
- **Loading Animations**: Visual feedback during video loading
- **File Browser**: Native file selection dialog
- **URL Input**: Direct video URL support

## Quick Start

### Prerequisites

**Rust**: Install from [rustup.rs](https://rustup.rs/)

**FFmpeg Development Libraries**:
```bash
# Linux (Debian/Ubuntu)
sudo apt update
sudo apt install clang libavcodec-dev libavformat-dev libavutil-dev libavfilter-dev libavdevice-dev libasound2-dev pkg-config

# macOS (Homebrew)
brew install pkg-config ffmpeg

# Windows (vcpkg)
vcpkg install ffmpeg --triplet x64-windows
```

### Installation & Usage

1. **Clone and Build**:
```bash
git clone https://github.com/JeninSutradhar/ffmpeg-VideoPlayer-Rust_Slint
cd ffmpeg-VideoPlayer-Rust_Slint
cargo build --release
```

2. **Run the Player**:
```bash
cargo run --release
# or
./target/release/ffmpeg
```

3. **Load Content**:
   - **From URL**: Paste video URL and click "Load URL"
   - **From File**: Click "Browse" to select local video file

## How to Use

### Loading Videos
1. **URL Playback**: 
   - Paste any video URL in the input field
   - Click "Load URL" to start playback
   - Supports YouTube, direct video links, etc.

2. **Local Files**:
   - Click "📁 Browse" button
   - Select video file from file dialog
   - Video starts playing automatically

### Playback Controls
- **▶ Play/⏸ Pause**: Toggle playback
- **⏹ Stop**: Stop video and reset
- **Seek Bar**: Drag to jump to any position
- **🔊 Volume**: Adjust audio level (0-100%)
- **Time Display**: Shows current/total time

## 🛠️ Technical Details

### Architecture
- **FFmpeg**: Video/audio decoding and processing
- **Slint UI**: Modern, cross-platform user interface
- **CPAL**: Cross-platform audio output
- **Ring Buffer**: Smooth audio streaming
- **Threading**: Separate threads for UI, video, and audio processing

### Performance
- **Hardware Acceleration**: Uses FFmpeg's optimized decoders
- **Memory Efficient**: Streaming playback with minimal memory usage
- **Real-time Processing**: Low-latency audio-video synchronization
- **Cross-platform**: Native performance on all supported platforms

## 🌍 Supported Platforms

- ✅ **Linux** (Tested)
- ✅ **Windows** 
- ✅ **macOS**
- 🔄 **Android** (Planned)
- 🔄 **WebAssembly** (Planned)

## 📋 Supported Formats

### Video Formats
- MP4, MKV, AVI, MOV, WebM, FLV, WMV, M4V
- H.264, H.265, VP8, VP9, AV1
- Any format supported by FFmpeg

### Audio Formats
- AAC, MP3, FLAC, OGG, WAV
- Any audio codec supported by FFmpeg

## 🔧 Development

### Building from Source
```bash
# Debug build
cargo build

# Release build (recommended)
cargo build --release

# Run tests
cargo test
```

### Dependencies
- `ffmpeg-next`: FFmpeg bindings for Rust
- `slint`: Modern UI framework
- `cpal`: Cross-platform audio library
- `ringbuf`: Lock-free ring buffer for audio
- `rfd`: Native file dialogs

## 🐛 Known Issues

- Seeking functionality is implemented but may need refinement
- Some exotic video formats may not work perfectly
- WebAssembly support is planned but not yet implemented

## 🤝 Contributing

Contributions are welcome! Please feel free to:
- Report bugs and issues
- Suggest new features
- Submit pull requests
- Improve documentation

## 📄 License

This project is licensed under the MIT License - see the LICENSE file for details.

## 🙏 Acknowledgments

- [FFmpeg](https://ffmpeg.org/) - The multimedia framework
- [Slint](https://slint-ui.com/) - The UI framework
- [CPAL](https://github.com/RustAudio/cpal) - Cross-platform audio library
- [Rust Community](https://www.rust-lang.org/community) - For the amazing ecosystem

---

**Made with ❤️ in Rust**
