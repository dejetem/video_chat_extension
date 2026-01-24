# API Reference

This document describes the external-facing APIs of the Video Chat Extension.

## WASM Interface (JavaScript to Rust)

These functions are exposed to JavaScript via `wasm-bindgen`.

### `init()`
Initializes the WASM module, logging, and the SQLite database. Must be called once before any other function.

### `create_room(room_id: String)`
Initializes a new room on the signaling and media layers. Dispatches a `roomCreated` event.

### `join_room(room_id: String)`
Joins an existing room. Dispatches a `roomJoined` event.

### `toggle_microphone(enabled: bool)`
Enables or disables local audio input. Dispatches a `mediaStatus` event.

### `toggle_camera(enabled: bool)`
Enables or disables local video input. Dispatches a `mediaStatus` event.

### `send_message(text: String)`
Sends a chat message to all participants in the current room via the DataChannel.

## Signaling Protocol (WebSocket)

Messages are JSON-encoded.

### Client to Server

- `JoinRoom { room_id, participant_id }`: Request to join a room.
- `Signal { room_id, participant_id, data }`: Forward a WebRTC signal (SDP or ICE candidate).

### Server to Client

- `RoomJoined { participants }`: Confirmation of joining a room.
- `Signal { participant_id, data }`: A signal forwarded from another participant.

## Events (Rust to JavaScript)

Dispatched via `CustomEvent` in the browser.

- `roomCreated`: Fired when a room is successfully created.
- `roomJoined`: Fired when joining a room is confirmed.
- `mediaStatus`: Fired when the mic or cam state changes.
- `messageReceived`: Fired when a chat message arrives.
- `messageSent`: Fired when a message is successfully sent.
