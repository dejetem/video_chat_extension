// Content script for Rust Video Chat Extension
console.log("Rust Video Chat content script loaded");

// Handle messages from the background script or popup
chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
    if (request.action === "toggleOverlay") {
        const overlay = document.getElementById('rust-video-chat-overlay');
        if (overlay) {
            overlay.style.display = overlay.style.display === 'none' ? 'block' : 'none';
        } else {
            createOverlay();
        }
        sendResponse({ status: "success" });
    }
});

function createOverlay() {
    const overlay = document.createElement('div');
    overlay.id = 'rust-video-chat-overlay';
    overlay.style.cssText = `
        position: fixed;
        bottom: 20px;
        right: 20px;
        width: 300px;
        height: 200px;
        background: #1a1a1e;
        border: 1px solid #333;
        border-radius: 12px;
        box-shadow: 0 8px 32px rgba(0,0,0,0.5);
        z-index: 10000;
        display: flex;
        align-items: center;
        justify-content: center;
        color: #f0f0f0;
        font-family: sans-serif;
    `;
    overlay.innerHTML = '<div>Video Chat Active</div>';
    document.body.appendChild(overlay);
}
