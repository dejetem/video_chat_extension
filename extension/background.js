// Background Service Worker for Rust Video Chat
console.log("Video Chat Service Worker Initialized");

async function ensureOffscreen() {
    if (await chrome.offscreen.hasDocument()) {
        return;
    }

    await chrome.offscreen.createDocument({
        url: 'offscreen.html',
        reasons: ['USER_MEDIA', 'WEB_RTC'],
        justification: 'Hosting WASM/WebRTC stack for real-time video calls (not available in SW)'
    });
    console.log("Offscreen document created");
}

// Proxy messages to offscreen context
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    // Skip if it's already targeted at offscreen
    if (request.target === "offscreen") return;

    console.log("SW proxying message to offscreen:", request.action);

    (async () => {
        try {
            await ensureOffscreen();
            // Wrap original message to target offscreen
            const offscreenMsg = { ...request, target: "offscreen" };
            const response = await chrome.runtime.sendMessage(offscreenMsg);
            sendResponse(response);
        } catch (e) {
            console.error("SW failed to proxy message:", e);
            sendResponse({ status: "error", message: e.message });
        }
    })();

    return true; // Keep channel open
});

chrome.runtime.onInstalled.addListener(() => {
    console.log("Video Chat Extension Installed");
});
