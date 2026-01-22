import init, {
    init as initWasm,
    create_room,
    join_room,
    toggle_microphone,
    toggle_camera,
    send_message
} from './pkg/video_chat_wasm.js';

// Initialize WASM when the service worker starts
async function run() {
    try {
        await init();
        initWasm();
        console.log("Rust WASM initialized in Service Worker");
    } catch (e) {
        console.error("Failed to initialize Rust WASM:", e);
    }
}

// Listen for messages from popup or content scripts
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    switch (request.action) {
        case "createRoom":
            create_room(request.roomId);
            sendResponse({ status: "success" });
            break;
        case "joinRoom":
            join_room(request.roomId);
            sendResponse({ status: "success" });
            break;
        case "toggleMic":
            toggle_microphone(request.enabled);
            sendResponse({ status: "success" });
            break;
        case "toggleCam":
            toggle_camera(request.enabled);
            sendResponse({ status: "success" });
            break;
        case "sendMessage":
            send_message(request.text);
            sendResponse({ status: "success" });
            break;
    }
    return true; // Keep message channel open for async response
});

chrome.runtime.onInstalled.addListener(() => {
    console.log("Video Chat Extension Installed");
});

run();
