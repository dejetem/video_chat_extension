# User Guide

Welcome to the Rust Video Chat Extension! This guide will help you get started with the peer-to-peer video conferencing features.

## Getting Started

1.  **Install the Extension**: Load the `extension/` directory into your Chromium-based browser (Chrome, Edge, Brave) via `chrome://extensions` with Developer Mode enabled.
2.  **Open the Extension**: Click the extension icon in your toolbar to open the popup.
3.  **Permissions**: Grant camera and microphone permissions when prompted.

## Creating and Joining Rooms

- **Join a Room**: Enter a Room ID in the input field and click "Join Room". If the room doesn't exist, it will be created automatically.
- **Sharing**: Share your Room ID with others so they can join the same session.

## Controls

- **Microphone**: Toggle your audio on and off using the microphone button.
- **Camera**: Toggle your video stream using the camera button.
- **Chat**: Use the chat panel on the right to send messages to everyone in the room.

## Performance and Quality

- **Bitrate**: The extension automatically adjusts video quality based on your network conditions.
- **SFU**: For sessions with more than two participants, the media streams are routed through a high-performance SFU server to ensure smooth playback.

## Troubleshooting

- **No Media**: Ensure your camera and microphone are not being used by another application.
- **Connection Failed**: Check your internet connection and ensure that you are not behind a restrictive firewall that blocks WebRTC traffic.
- **Extension not loading**: Try reloading the extension in the `chrome://extensions` page.
