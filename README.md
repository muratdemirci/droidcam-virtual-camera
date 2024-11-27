# DroidCam Virtual Camera

DroidCam Virtual Camera is a Rust-based application designed to display live camera streams using the `egui` GUI framework. It connects to a remote video stream URL, decodes incoming frames in real-time, and renders them in a user-friendly interface.

---

## Features

- **Real-Time Video Streaming:** Streams video from a specified URL.
- **Automatic Reconnection:** Automatically attempts to reconnect if the connection drops.
- **Efficient Frame Decoding:** Processes JPEG frames from a multipart stream efficiently.
- **Customizable UI:** Leverages the `eframe` and `egui` frameworks for a responsive and modern user interface.
- **Performance Optimized:** Supports TCP keep-alive and adjustable connection timeouts for optimal performance.

---

## Getting Started

### Prerequisites

- **Rust Toolchain**: Ensure that Rust and Cargo are installed. [Install Rust](https://www.rust-lang.org/tools/install)
- **Dependencies**: The project relies on the following crates:
  - `eframe` and `egui` for GUI.
  - `reqwest` for HTTP requests.
  - `tokio` for asynchronous runtime.
  - `image` for image processing.
  - `futures-util` for handling asynchronous streams.

### Installation

1. Clone the repository:

   ```bash
   git clone https://github.com/muratdemirci/droidcam-virtual-camera.git
   cd droidcam-virtual-camera
   ```

2. Install dependencies:

   ```bash
   cargo build
   ```

3. Run the application:
   ```bash
   cargo run
   ```

---

## Configuration

### Default Stream URL

The application connects to the following default stream URL:

```plaintext
http://192.168.0.101:4747/video
```

You can modify the `url` field in the `App` struct to connect to a different stream.

---

## Usage

1. **Start the Application**: Launch the app using `cargo run`.
2. **View the Stream**: If the stream is reachable, the application will display the video frames in real-time.
3. **Reconnect Logic**: If the connection drops, the app will retry with an exponential backoff mechanism.
4. **Adjust Window Size**: Resize the application window to fit your screen.

---

## Project Structure

- **Main Application (`App`)**:
  - Manages the video stream, connection logic, and frame decoding.
- **Frame Decoding**:
  - Extracts JPEG frames from a multipart stream.
- **User Interface**:
  - Displays video frames using `egui`'s image widget.
- **Stream Task**:
  - Runs in a separate asynchronous task for continuous streaming.

---

## Troubleshooting

- **Connection Issues**:
  - Ensure the URL is correct and reachable.
  - Verify the device providing the stream is active.
- **Decoding Errors**:
  - Check the stream format; the application expects JPEG frames.
  - Review error messages for specific issues with data format.

---

## Future Enhancements

- Support for multiple camera streams.
- Adjustable resolution and quality settings.
- Integration with additional video formats (e.g., MJPEG).
- Improved error handling and diagnostics.

---

## License

This project is licensed under the MIT License. See the `LICENSE` file for details.

---

## Contribution

Contributions are welcome! Feel free to open issues or submit pull requests.

---

## Author

Developed by **[Murat Demirci]**.
