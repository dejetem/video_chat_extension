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

    // Open Full-Page Web Interface
    const toggleScreen = document.getElementById('toggleScreen');
    const roomIdInput = document.getElementById('roomIdInput');
    const joinRoomBtn = document.getElementById('joinRoomBtn');

    // Fetch tunnel URLs from server config
    let tunnelConfig = {
        tunnelUrl: 'http://localhost:8080',
        tunnelWsUrl: 'ws://localhost:8080/ws'
    };

    // Try to fetch config from server
    fetch('http://localhost:8080/api/config')
        .then(res => res.json())
        .then(config => {
            tunnelConfig.tunnelUrl = config.tunnelUrl;
            tunnelConfig.tunnelWsUrl = config.tunnelWsUrl;
            console.log('Loaded tunnel config:', tunnelConfig);
        })
        .catch(err => {
            console.warn('Failed to load config, using defaults:', err);
        });

    joinRoomBtn.addEventListener('click', () => {
        const roomId = roomIdInput.value.trim() || "default-room";

        statusEl.textContent = "Joining...";

        chrome.runtime.sendMessage({
            action: "joinRoom",
            roomId: roomId,
            signalingUrl: tunnelConfig.tunnelWsUrl
        }, (response) => {
            if (response && response.status === "success") {
                addMessage(`Joined room: ${roomId}`, 'received');
            } else {
                addMessage(`Failed to join: ${response ? response.message : 'Unknown error'}`, 'received');
            }
        });
    });

    toggleScreen.addEventListener('click', () => {
        const roomId = roomIdInput.value.trim() || "default-room";
        const externalUrl = `${tunnelConfig.tunnelUrl}/index.html?room=${roomId}`;
        window.open(externalUrl, '_blank');
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

    // Ready
    console.log("Popup ready for room interaction");
});
