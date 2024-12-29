Okay, that .gitignore looks perfect now. You've correctly removed the exclusion of .rs and .toml files, ensuring that your source code and project configuration are properly tracked by Git.

Now, let's craft a user-friendly README that explains how to use your video player, incorporates a warning message, and includes a clear explanation for how to run the project.

Here's an improved README.md content that you can use:

# FFmpeg Rust Video Player

A lightweight audio-video player built in Rust using FFmpeg and Slint UI libraries. This project demonstrates how to use FFmpeg with Rust to play back video files in a simple user interface.

⚠️ **Warning: Under Development** ⚠️

This video player is currently under active development and may have bugs or incomplete features. We are working on improvements and will address known issues soon.

## Features

*   **Simple Video Playback:** Plays video files using the power of FFmpeg.
*   **Basic UI:** Has a simple user interface for basic video playback control using Slint UI.
*   **Cross-Platform:** Designed to be compatible with multiple platforms (Android, Windows, Linux, macOS, and WebAssembly), though testing and configuration may vary.
*   **Audio Playback:** It plays audio from a video file and the audio data is streamed to the system's audio device.

## Prerequisites

Before building and running the application, ensure you have the following installed on your system:

*   **Rust:** Install Rust from [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
*   **FFmpeg:** Install FFmpeg along with its development headers using your operating system's package manager:
    *   **Linux (Debian/Ubuntu):**
        ```bash
        sudo apt update
        sudo apt install clang libavcodec-dev libavformat-dev libavutil-dev libavfilter-dev libavdevice-dev libasound2-dev pkg-config
        ```
    *   **macOS (Homebrew):**
        ```bash
        brew install pkg-config ffmpeg
        ```
    *   **Windows:** Install `vcpkg`. Then run:
        ```bash
        vcpkg install ffmpeg --triplet x64-windows
        ```
        Make sure `VCPKG_ROOT` is set to where vcpkg is installed, and that `%VCPKG_ROOT%\installed\x64-windows\bin` is in your PATH.
    * **Other Systems:** Install ffmpeg and its development packages using your system's package manager.

*   **Clang:**  The Clang compiler is required to build `ffmpeg-sys-next`. It is usually included in the default installation of operating systems or can be installed through the OS package manager.

## Building the Application

To build the application, follow these steps:

1.  **Clone the repository:**
    ```bash
    git clone https://github.com/JeninSutradhar/ffmpeg-VideoPlayer-Rust_Slint
    cd ffmpeg-VideoPlayer-Rust_Slint
    ```

2.  **Build using `cargo`:**
    ```bash
    cargo build --release
    ```
    This will compile your project into a release executable.

## Running the Application

1.  **Navigate to the directory** Inside your project root, run the command:
   ```bash
   cargo run --release

- This will execute the application and open up a window where you can select a video to be played.


## Supported Platforms
This application is designed to be compatible with the following platforms:

1. Android
2. Windows
3. Linux
4. macOS
5. WebAssembly (wasm32)

Note that testing and configuration for WebAssembly may require additional setup and might not be fully supported.
Also, the project has been tested only on Linux.

## Known Issues and Future Improvements

- The audio and video playback may not be fully synchronized.
- The user interface is minimal and may have some limitations.
- The application may not be stable in all platforms.
We are planning to address these issues in upcoming updates.

## Contributing
Contributions are welcome! Feel free to submit issues and pull requests.

## License
This project is licensed under the MIT License. (If you want to include a license)

