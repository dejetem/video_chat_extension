import init, {
    init as initWasm,
    create_room,
    join_room,
    leave_room,
    toggle_microphone,
    toggle_camera,
    send_message,
    add_track,
    add_stream,
    attach_remote_track
} from './pkg/video_chat_wasm.js';

let wasmInitialized = false;

document.addEventListener('DOMContentLoaded', async () => {
    console.log('Web Video Chat Initialized');

    // UI Elements
    const videoGrid = document.getElementById('videoGrid');
    const localVideo = document.getElementById('localVideo');
    const toggleMic = document.getElementById('toggleMic');
    const toggleCam = document.getElementById('toggleCam');
    const leaveCall = document.getElementById('leaveCall');
    const shareLink = document.getElementById('shareLink');
    const chatInput = document.getElementById('chatInput');
    const sendMsg = document.getElementById('sendMsg');
    const chatMessages = document.getElementById('chatMessages');
    const roomBadge = document.getElementById('roomBadge');
    const loadingOverlay = document.getElementById('loadingOverlay');
    const loadingStatus = document.getElementById('loadingStatus');

    // State
    const urlParams = new URLSearchParams(window.location.search);
    let roomId = urlParams.get('room') || 'default-room';
    let micOn = true;
    let camOn = true;

    // 1. Initialize WASM
    try {
        await init();
        initWasm();
        wasmInitialized = true;
        console.log('Rust WASM loaded successfully');

        loadingStatus.textContent = `Joining Room: ${roomId}...`;

        // Use the Cloudflare tunnel URL for signaling
        const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
        const signalingUrl = `${protocol}//${window.location.host}/ws`;

        // Join the room
        join_room(roomId, signalingUrl);
        roomBadge.textContent = `ROOM: ${roomId.toUpperCase()}`;

        // Setup Local Media
        setupLocalMedia();

        // Hide loading after a bit
        setTimeout(() => {
            loadingOverlay.style.opacity = '0';
            setTimeout(() => loadingOverlay.style.display = 'none', 500);
        }, 1500);

    } catch (e) {
        console.error('Failed to init WASM:', e);
        loadingStatus.textContent = 'Failed to connect to SFU. Please try again.';
    }

    // 2. Media Setup
    async function setupLocalMedia() {
        try {
            const stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
            localVideo.srcObject = stream;

            if (wasmInitialized) {
                add_stream(stream);
            }
        } catch (e) {
            console.warn('Media access denied:', e);
        }
    }

    // 3. Event Listeners
    toggleMic.addEventListener('click', () => {
        micOn = !micOn;
        toggleMic.classList.toggle('active', micOn);
        toggleMic.textContent = micOn ? '🎤' : '🔇';
        if (wasmInitialized) toggle_microphone(micOn);
    });

    toggleCam.addEventListener('click', () => {
        camOn = !camOn;
        toggleCam.classList.toggle('active', camOn);
        toggleCam.textContent = camOn ? '📷' : '🚫';
        if (wasmInitialized) toggle_camera(camOn);
    });

    shareLink.addEventListener('click', async () => {
        const link = window.location.href;
        try {
            await navigator.clipboard.writeText(link);
            const originalText = shareLink.innerHTML;
            shareLink.innerHTML = '<span>Copied!</span>';
            setTimeout(() => shareLink.innerHTML = originalText, 2000);
        } catch (err) {
            console.error('Failed to copy link:', err);
        }
    });

    leaveCall.addEventListener('click', () => {
        if (confirm('Are you sure you want to leave the call?')) {
            if (wasmInitialized) leave_room();
            window.location.reload(); // Go back to join state
        }
    });

    // 4. Chat logic
    const addMessage = (text, sender = 'You') => {
        const msgDiv = document.createElement('div');
        msgDiv.className = 'message';
        msgDiv.innerHTML = `
            <div style="font-size: 0.75rem; color: var(--text-secondary); margin-bottom: 4px;">${sender}</div>
            <div class="message-content">${text}</div>
        `;
        chatMessages.appendChild(msgDiv);
        chatMessages.scrollTop = chatMessages.scrollHeight;
    };

    const handleSend = () => {
        const text = chatInput.value.trim();
        if (text) {
            if (wasmInitialized) {
                send_message(text);
                // Don't add message locally - wait for server broadcast
                // This prevents duplication
                chatInput.value = '';
            }
        }
    };

    sendMsg.addEventListener('click', handleSend);
    chatInput.addEventListener('keypress', (e) => {
        if (e.key === 'Enter') handleSend();
    });

    // 5. Screen Share
    const toggleScreen = document.getElementById('toggleScreen');
    let screenStream = null;
    toggleScreen.addEventListener('click', async () => {
        try {
            if (!screenStream) {
                screenStream = await navigator.mediaDevices.getDisplayMedia({ video: true });
                toggleScreen.classList.add('active');
                addMessage("Started screen sharing", "System");

                if (wasmInitialized) {
                    add_stream(screenStream);
                }

                screenStream.getVideoTracks()[0].onended = () => {
                    screenStream = null;
                    toggleScreen.classList.remove('active');
                    addMessage("Stopped screen sharing", "System");
                };
            } else {
                screenStream.getTracks().forEach(t => t.stop());
                screenStream = null;
                toggleScreen.classList.remove('active');
            }
        } catch (err) {
            console.error("Screen share failed:", err);
        }
    });

    // 6. Incoming Events from Rust WASM
    window.addEventListener('rust-video-chat-event', (event) => {
        const { event: type, data } = event.detail;
        console.log("Web portal received WASM event:", type, data);

        switch (type) {
            case "messageReceived":
                // Data format is "sender:text"
                const colonIndex = data.indexOf(':');
                if (colonIndex > 0) {
                    const sender = data.substring(0, colonIndex);
                    const text = data.substring(colonIndex + 1);
                    addMessage(text, sender);
                } else {
                    // Fallback if format is unexpected
                    addMessage(data, "Participant");
                }
                break;
            case "trackReceived":
                handleTrackReceived(data);
                break;
            case "roomJoined":
                console.log("Successfully joined room:", data);
                roomBadge.textContent = `ROOM: ${data.toUpperCase()}`;
                break;
            case "mediaStatus":
                console.log("Peer media update:", data);
                break;
            case "roomLeft":
                console.log("Left room confirmed");
                break;
            case "logsCleared":
                console.log("Logs cleared globally for room:", data);
                chatMessages.innerHTML = '';
                addMessage("Session ended. All chat logs have been cleared.", "System");
                break;
        }
    });

    function handleTrackReceived(trackId) {
        console.log("Handling remote track:", trackId);

        // Determine if this is a screen share based on track ID or label
        // Screen share tracks typically have "screen" in their ID
        const isScreenShare = trackId.toLowerCase().includes('screen');

        const videoCard = document.createElement('div');
        videoCard.className = isScreenShare ? 'video-card screen-share-box' : 'video-card remote';
        const videoId = `remote-video-${trackId}`;
        videoCard.innerHTML = `
            <video id="${videoId}" autoplay playsinline></video>
            <div class="participant-name">${isScreenShare ? 'Shared Screen' : 'Participant'}</div>
        `;

        if (isScreenShare) {
            // Screen shares go in a small floating box
            document.body.appendChild(videoCard);
        } else {
            // Regular participant videos go in the grid
            videoGrid.appendChild(videoCard);
        }

        // Attach track to video element
        attach_remote_track(videoId, trackId);
    }
});
