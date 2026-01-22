document.addEventListener('DOMContentLoaded', () => {
    console.log('Rust Video Chat Popup Initialized');

    const toggleMic = document.getElementById('toggleMic');
    const toggleCam = document.getElementById('toggleCam');
    const sendBtn = document.getElementById('sendBtn');
    const messageInput = document.getElementById('messageInput');
    const chatArea = document.getElementById('chatArea');
    const statusEl = document.getElementById('connectionStatus');

    let micOn = true;
    let camOn = true;

    toggleMic.addEventListener('click', () => {
        micOn = !micOn;
        toggleMic.classList.toggle('active', micOn);
        chrome.runtime.sendMessage({ action: "toggleMic", enabled: micOn });
    });

    toggleCam.addEventListener('click', () => {
        camOn = !camOn;
        toggleCam.classList.toggle('active', camOn);
        chrome.runtime.sendMessage({ action: "toggleCam", enabled: camOn });
    });

    const addMessage = (text, type = 'sent') => {
        const msgDiv = document.createElement('div');
        msgDiv.className = `message ${type}`;
        msgDiv.textContent = text;
        chatArea.appendChild(msgDiv);
        chatArea.scrollTop = chatArea.scrollHeight;
    };

    sendBtn.addEventListener('click', () => {
        const text = messageInput.value.trim();
        if (text) {
            chrome.runtime.sendMessage({ action: "sendMessage", text: text }, (response) => {
                if (response && response.status === "success") {
                    addMessage(text, 'sent');
                    messageInput.value = '';
                }
            });
        }
    });

    messageInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter' && !e.shiftKey) {
            e.preventDefault();
            sendBtn.click();
        }
    });

    // Listen for events from background/WASM
    chrome.runtime.onMessage.addListener((request, sender, sendResponse) => {
        if (request.event === "roomJoined") {
            statusEl.textContent = "Connected";
            statusEl.style.color = "#10b981";
        } else if (request.event === "messageSent") {
            // Confirm message sent or handle incoming
            console.log("WASM confirmed message sent:", request.data);
        } else if (request.event === "mediaStatus") {
            console.log("WASM media status update:", request.data);
        }
    });

    // Initial check for room
    chrome.runtime.sendMessage({ action: "joinRoom", roomId: "default-room" });
});
