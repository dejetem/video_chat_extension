// Background Service Worker
// Initializes the Rust WASM module

import init, { init as initWasm } from './pkg/video_chat_wasm.js';

// Initialize WASM when the service worker starts
async function run() {
    try {
        await init();
        // Call the Rust init function (renamed to avoid conflict with default export)
        initWasm();
        console.log("Rust WASM initialized in Service Worker");
    } catch (e) {
        console.error("Failed to initialize Rust WASM:", e);
    }
}

// Listen for installation
chrome.runtime.onInstalled.addListener(() => {
    console.log("Video Chat Extension Installed");
});

run();
