import init, {
    init as initWasm,
    create_room,
    join_room,
    toggle_microphone,
    toggle_camera,
    send_message
} from './pkg/video_chat_wasm.js';

console.log("Offscreen Media Bridge Loaded");

console.log("Offscreen Media Bridge Loaded");

// Create a promise that resolves when WASM is fully initialized
const initPromise = (async () => {
    try {
        await init();
        initWasm();
        console.log("WASM initialized in Offscreen Document");
        return true;
    } catch (e) {
        console.error("Failed to init WASM in offscreen:", e);
        throw e;
    }
})();

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
    // Only handle messages meant for offscreen
    if (message.target !== "offscreen") return;

    console.log("Offscreen handling action:", message.action);

    // Use an async IIFE to await initialization
    (async () => {
        try {
            await initPromise;

            switch (message.action) {
                case "createRoom":
                    create_room(message.roomId, message.signalingUrl);
                    sendResponse({ status: "success" });
                    break;
                case "joinRoom":
                    join_room(message.roomId, message.signalingUrl);
                    sendResponse({ status: "success" });
                    break;
                case "toggleMic":
                    toggle_microphone(message.enabled);
                    sendResponse({ status: "success" });
                    break;
                case "toggleCam":
                    toggle_camera(message.enabled);
                    sendResponse({ status: "success" });
                    break;
                case "sendMessage":
                    send_message(message.text);
                    sendResponse({ status: "success" });
                    break;
                default:
                    sendResponse({ status: "error", message: `Unknown action: ${message.action}` });
            }
        } catch (e) {
            console.error("Offscreen error handling action:", e);
            sendResponse({ status: "error", message: e.message });
        }
    })();

    return true; // Keep channel open for async response
});
