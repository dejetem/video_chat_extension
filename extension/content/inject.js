// Inject script to run in the page context if needed
// For now, it provides helper functions for content scripts

window.__RUST_VIDEO_CHAT__ = {
    showOverlay: () => {
        console.log("Showing video chat overlay");
    },
    hideOverlay: () => {
        console.log("Hiding video chat overlay");
    }
};
