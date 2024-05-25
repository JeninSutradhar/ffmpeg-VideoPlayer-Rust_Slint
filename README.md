# FFmpeg Rust Video Player

A lightweight audio-video player built in Rust using FFmpeg libraries. This project demonstrates how to use FFmpeg with Rust to play back video files.

![image](https://github.com/JeninSutradhar/ffmpeg-VideoPlayer-Rust_Slint/assets/111521642/8507dbad-fb04-4dab-a696-d52ab95159c8)


### Supported Builds -
- Android
- Windows
- Linux
- Mac
- (wasm32)

## Necessary Libraries
- Before building the application, you need to install the necessary libraries for your platform.

This example application requires the following libraries:

### Linux

- On Linux, you need to install FFmpeg and ALSA. For example, on Debian-based systems:

```bash
sudo apt-get install clang libavcodec-dev libavformat-dev libavutil-dev libavfilter-dev libavdevice-dev libasound2-dev pkg-config
```

- macOS
On macOS, you can use Homebrew:
```bash
brew install pkg-config ffmpeg
```

- Windows
For Windows:
```bash
Install vcpkg.
Run vcpkg install ffmpeg --triplet x64-windows.
Make sure VCPKG_ROOT is set to where vcpkg is installed.
Make sure %VCPKG_ROOT%\installed\x64-windows\bin is in your PATH.
```

- WebAssembly (wasm32)
For WebAssembly, additional setup might be required. Please refer to relevant documentation for building Rust applications with WebAssembly.

# Building the Application
To build the application, follow these steps:

- Ensure you have installed the necessary libraries as mentioned above.
- Navigate to the root directory of the project.
- Run the following command:
```bash
cargo bundle --release
```
This command bundles the application along with its dependencies into a single distributable directory.

## Project Structure
```bash
└── ffmpeg
    ├── Cargo.toml
    ├── README.md
    ├── api
    │   └── rs
    │       ├── build
    │       │   ├── Cargo.toml
    │       │   ├── LICENSES
    │       │   │   ├── GPL-3.0-only.txt -> ../../../../LICENSES/GPL-3.0-only.txt
    │       │   │   ├── LicenseRef-Slint-Royalty-free-1.1.md -> ../../../../LICENSES/LicenseRef-Slint-Royalty-free-1.1.md
    │       │   │   └── LicenseRef-Slint-commercial.md -> ../../../../LICENSES/LicenseRef-Slint-commercial.md
    │       │   └── lib.rs
    │       ├── macros
    │       │   ├── Cargo.toml
    │       │   ├── LICENSES
    │       │   │   ├── GPL-3.0-only.txt -> ../../../../LICENSES/GPL-3.0-only.txt
    │       │   │   ├── LicenseRef-Slint-Royalty-free-1.1.md -> ../../../../LICENSES/LicenseRef-Slint-Royalty-free-1.1.md
    │       │   │   └── LicenseRef-Slint-commercial.md -> ../../../../LICENSES/LicenseRef-Slint-commercial.md
    │       │   ├── README.md
    │       │   └── lib.rs
    │       └── slint
    │           ├── Cargo.toml
    │           ├── LICENSES
    │           │   ├── GPL-3.0-only.txt -> ../../../../LICENSES/GPL-3.0-only.txt
    │           │   ├── LicenseRef-Slint-Royalty-free-1.1.md -> ../../../../LICENSES/LicenseRef-Slint-Royalty-free-1.1.md
    │           │   ├── LicenseRef-Slint-commercial.md -> ../../../../LICENSES/LicenseRef-Slint-commercial.md
    │           │   └── MIT.txt -> ../../../../LICENSES/MIT.txt
    │           ├── README.md
    │           ├── android.rs
    │           ├── compile_fail_tests.rs
    │           ├── docs -> ../../../docs
    │           ├── docs.rs
    │           ├── lib.rs
    │           ├── mcu.md
    │           ├── private_unstable_api.rs
    │           ├── tests
    │           │   ├── partial_renderer.rs
    │           │   ├── show_strongref.rs
    │           │   ├── simple_macro.rs
    │           │   └── spawn_local.rs
    │           └── type-mappings.md
    ├── build.rs
    ├── main.rs
    ├── pause.svg
    ├── play.svg
    ├── player
    │   ├── audio.rs
    │   └── video.rs
    ├── player.rs
    └── scene.slint (UI) 
```
