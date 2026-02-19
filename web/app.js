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
        // Explicitly load WASM with cache busting to prevent stale binary
        await init(`./pkg/video_chat_wasm_bg.wasm?v=${Date.now()}`);
        initWasm();
        wasmInitialized = true;
        console.log('Rust WASM loaded successfully');

        loadingStatus.textContent = `Joining Room: ${roomId}...`;

        // TEMPORARY: Force localhost for testing (bypass Cloudflare tunnel)
        // const signalingUrl = 'ws://localhost:8080/ws';
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
        let stream = null;

        // Try different media configurations in order of preference
        try {
            // Try video + audio first
            stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: true });
            console.log('Got video + audio');
        } catch (e) {
            console.warn('Could not get video + audio:', e.message);

            try {
                // Try audio only
                stream = await navigator.mediaDevices.getUserMedia({ video: false, audio: true });
                console.log('Got audio only (no video)');
            } catch (e2) {
                console.warn('Could not get audio:', e2.message);

                try {
                    // Try video only
                    stream = await navigator.mediaDevices.getUserMedia({ video: true, audio: false });
                    console.log('Got video only (no audio)');
                } catch (e3) {
                    console.warn('Could not get video:', e3.message);
                    console.warn('No media devices available - continuing without local media');

                    // Hide local video container if no media
                    const localContainer = document.getElementById('localVideoContainer');
                    if (localContainer) {
                        localContainer.style.display = 'none';
                    }
                    return; // Exit early - no media available
                }
            }
        }

        // If we got a stream, use it
        if (stream) {
            localVideo.srcObject = stream;

            if (wasmInitialized) {
                add_stream(stream);
            }
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
    let screenSharePreview = null; // Local preview element

    toggleScreen.addEventListener('click', async () => {
        try {
            if (!screenStream) {
                // Get screen share stream
                screenStream = await navigator.mediaDevices.getDisplayMedia({ video: true });
                toggleScreen.classList.add('active');
                addMessage("Started screen sharing", "System");

                // Create local preview for screen share
                screenSharePreview = document.createElement('div');
                screenSharePreview.className = 'video-card screen-share-box';
                screenSharePreview.id = 'local-screen-share';
                screenSharePreview.innerHTML = `
                    <video id="localScreenVideo" autoplay playsinline muted></video>
                    <div class="participant-name">Your Shared Screen</div>
                `;
                document.body.appendChild(screenSharePreview);

                // Attach stream to preview
                const localScreenVideo = document.getElementById('localScreenVideo');
                localScreenVideo.srcObject = screenStream;

                // Send screen share to other participants
                if (wasmInitialized) {
                    add_stream(screenStream);
                }

                // Handle when user stops sharing via browser UI
                screenStream.getVideoTracks()[0].onended = () => {
                    screenStream = null;
                    toggleScreen.classList.remove('active');
                    addMessage("Stopped screen sharing", "System");

                    // Remove local preview
                    if (screenSharePreview) {
                        screenSharePreview.remove();
                        screenSharePreview = null;
                    }
                };
            } else {
                // Stop screen sharing
                screenStream.getTracks().forEach(t => t.stop());
                screenStream = null;
                toggleScreen.classList.remove('active');
                addMessage("Stopped screen sharing", "System");

                // Remove local preview
                if (screenSharePreview) {
                    screenSharePreview.remove();
                    screenSharePreview = null;
                }
            }
        } catch (err) {
            console.error("Screen share failed:", err);
            addMessage("Screen share failed: " + err.message, "System");
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

    function handleTrackReceived(data) {
        console.log("=== TRACK RECEIVED ===");

        let trackId = data;
        let participantId = null;
        let kind = 'video'; // Default

        // Parse "trackId|participantId|kind" format
        if (data.includes('|')) {
            const parts = data.split('|');
            trackId = parts[0];
            participantId = parts[1];
            if (parts.length > 2) {
                kind = parts[2];
            }
        }

        console.log("Track RECEIVED:", trackId, "Participant:", participantId, "Kind:", kind);

        // Detect Screen Share
        const isScreenShare = trackId.toLowerCase().includes('screen');

        // 1. Screen Share (Separate Box)
        if (isScreenShare) {
            console.log("Creating SCREEN SHARE element for track:", trackId);
            const videoId = `remote-video-${trackId}`;
            if (document.getElementById(videoId)) return;

            const videoCard = document.createElement('div');
            videoCard.className = 'video-card screen-share-box';
            videoCard.setAttribute('data-track-id', trackId);
            if (participantId) videoCard.setAttribute('data-participant-id', participantId);

            videoCard.innerHTML = `
                <video id="${videoId}" autoplay playsinline muted></video>
                <div class="participant-name">Shared Screen</div>
            `;
            document.body.appendChild(videoCard);
            attach_remote_track(videoId, trackId);
            return;
        }

        // 2. Audio Track (Hidden but Active)
        if (kind === 'audio') {
            const audioId = `remote-audio-${trackId}`;
            if (document.getElementById(audioId)) {
                console.log("Audio element already exists, skipping:", audioId);
                return;
            }

            console.log("Creating audio element for track:", trackId);
            const audioEl = document.createElement('audio');
            audioEl.id = audioId;
            audioEl.autoplay = true;

            // IMPORTANT: display:none can pause playback in some browsers.
            // Use 1px size and absolute position instead.
            audioEl.style.position = 'absolute';
            audioEl.style.width = '1px';
            audioEl.style.height = '1px';
            audioEl.style.opacity = '0.01';
            audioEl.style.pointerEvents = 'none';
            audioEl.style.zIndex = '-1';

            if (participantId) audioEl.setAttribute('data-participant-id', participantId);
            document.body.appendChild(audioEl);

            // Attach track to audio element
            if (typeof attach_remote_track === 'function') {
                attach_remote_track(audioId, trackId);
                // Force play after a short delay to ensure attached
                setTimeout(() => {
                    audioEl.play().catch(e => console.warn("Audio autoplay failed, waiting for user interaction:", e));
                }, 500);
            }
        }

        // 3. Manage Visual Participant Card (Unified)
        if (!participantId) {
            console.warn("No participant ID for track, cannot group elements");
            return;
        }

        const cardId = `participant-card-${participantId}`;
        let participantCard = document.getElementById(cardId);

        // Create Card if missing
        if (!participantCard) {
            console.log("Creating new Participant Card for:", participantId);
            participantCard = document.createElement('div');
            participantCard.id = cardId;
            participantCard.className = 'video-card remote';
            participantCard.setAttribute('data-participant-id', participantId);

            // Default: Audio Placeholder
            participantCard.innerHTML = `
                <div class="video-placeholder" id="placeholder-${participantId}" style="width:100%; height:100%; display:flex; flex-direction:column; align-items:center; justify-content:center; background:#222;">
                    <div class="avatar" style="font-size:3rem;">👤</div>
                    <div class="status" style="margin-top:10px; color:#aaa;">Audio Only</div>
                </div>
                <div class="participant-name">${participantId}</div>
            `;
            videoGrid.appendChild(participantCard);
        } else {
            console.log("Participant Card already exists for:", participantId);
        }

        if (kind === 'video') {
            const videoId = `remote-video-${trackId}`;
            console.log("Processing VIDEO track:", trackId, "for participant:", participantId);

            // QUANTUM FIX 2.0: Synchronous Memory-Based Deduplication
            // Prevents async race conditions where strict DOM check fails
            if (!window.activeVideoTracks) {
                window.activeVideoTracks = new Set();
            }

            if (window.activeVideoTracks.has(trackId)) {
                console.log("DEDUPE: Track already active in memory:", trackId);
                return;
            }

            // Check if THIS specific video track is already attached in DOM (fallback)
            if (document.getElementById(videoId)) {
                console.log("DEDUPE: Video element already exists for this track:", videoId);
                return;
            }

            // Mark as active IMMEDIATELY before awaits or DOM ops
            window.activeVideoTracks.add(trackId);

            // cleanup helper
            const cleanup = () => window.activeVideoTracks.delete(trackId);

            // Check if Card already has ANY video (avoid duplicate videos in one card)
            // If the card already has a video element, we should be very careful.
            // If the EXISTING video has the SAME track ID, do nothing.
            // If the EXISTING video has a DIFFERENT track ID, replace it.
            const existingVideo = participantCard.querySelector('video');
            if (existingVideo) {
                // Check if it's the same track ID attached
                if (existingVideo.id === videoId) {
                    console.log("Video element ALREADY exists and matches ID. Skipping duplicate creation.");
                    cleanup(); // It's already there, so we technically didn't add a NEW one, but let's keep set consistent? 
                    // Actually if it's already there, we should keep it in Set.
                    // But we are returning, so we didn't do anything.
                    return;
                }
                console.warn("Card has video, but ID mismatch. Replacing.", existingVideo.id, "with", videoId);
                // Remove the old track from the Set if it exists
                if (window.activeVideoTracks) {
                    const oldTrackId = existingVideo.id.replace('remote-video-', '');
                    window.activeVideoTracks.delete(oldTrackId);
                }
                existingVideo.remove();
            }


            // Remove placeholder
            const placeholder = document.getElementById(`placeholder-${participantId}`);
            if (placeholder) placeholder.remove();

            // Create Video Element
            const videoEl = document.createElement('video');
            videoEl.id = videoId;
            videoEl.autoplay = true;
            videoEl.playsInline = true;
            videoEl.muted = true; // Start muted to allow autoplay
            videoEl.style.width = '100%';
            videoEl.style.height = '100%';
            videoEl.style.objectFit = 'cover';

            // Insert video at top of card
            participantCard.insertBefore(videoEl, participantCard.firstChild);

            // Attach track
            attach_remote_track(videoId, trackId);

            // DEBUG: Monitor Video Stats
            const statsInterval = setInterval(() => {
                const v = document.getElementById(videoId);
                if (!v) {
                    clearInterval(statsInterval);
                    return;
                }
                if (v.videoWidth > 0 || v.readyState >= 2) {
                    console.log(`Video ${videoId} ALIVE: ${v.videoWidth}x${v.videoHeight}, State: ${v.readyState}, Paused: ${v.paused}, Muted: ${v.muted}`);
                    clearInterval(statsInterval);
                } else {
                    console.log(`Video ${videoId} WAITING: State: ${v.readyState}, Paused: ${v.paused}, NetState: ${v.networkState}`);
                }
            }, 2000);

            // Force play
            setTimeout(() => {
                videoEl.play().catch(e => console.warn("Video autoplay failed:", e));
            }, 500);

            // Cleanup listener
            videoEl.srcObject.getTracks().forEach(t => {
                t.onended = () => {
                    console.log("Video track ended:", trackId);
                    clearInterval(statsInterval);
                    videoEl.remove();
                    // If no video left, show placeholder?
                    if (!participantCard.querySelector('video')) {
                        const ph = document.createElement('div');
                        ph.className = 'video-placeholder';
                        ph.id = `placeholder-${participantId}`;
                        ph.style.cssText = "width:100%; height:100%; display:flex; flex-direction:column; align-items:center; justify-content:center; background:#222;";
                        ph.innerHTML = `
                            <div class="avatar" style="font-size:3rem;">👤</div>
                            <div class="status" style="margin-top:10px; color:#aaa;">Audio Only</div>
                        `;
                        participantCard.insertBefore(ph, participantCard.firstChild);
                    }
                };
            });
        }

        console.log("=== END TRACK RECEIVED ===");
    }

    // Handle participant disconnect - remove their video elements
    window.addEventListener('rust-video-chat-event', (event) => {
        const { event: type, data } = event.detail;

        if (type === "participantLeft") {
            const participantId = data;
            console.log("Participant left, removing video elements for:", participantId);

            // Remove all video elements for this participant
            const elementsToRemove = document.querySelectorAll(`[data-participant-id="${participantId}"]`);
            elementsToRemove.forEach(el => {
                console.log("Removing stale video element:", el);
                el.remove();
            });

            if (elementsToRemove.length === 0) {
                console.log("No elements found for participant:", participantId);
            }
        }
    });

    // Clean up on page unload
    window.addEventListener('beforeunload', () => {
        if (wasmInitialized) {
            leave_room();
        }
    });
});
