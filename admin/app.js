document.addEventListener('DOMContentLoaded', () => {
    console.log('Admin Dashboard Initialized');

    // Real Data Fetching
    async function fetchRooms() {
        try {
            const response = await fetch('/api/rooms'); // Assuming served by same server
            const rooms = await response.json();
            renderRooms(rooms);
        } catch (e) {
            console.error('Failed to fetch rooms:', e);
        }
    }

    const roomTableBody = document.getElementById('roomTableBody');
    const totalRoomsEl = document.getElementById('totalRooms');
    const totalParticipantsEl = document.getElementById('totalParticipants');

    function renderRooms(rooms) {
        roomTableBody.innerHTML = '';
        let totalParts = 0;

        rooms.forEach(room => {
            totalParts += room.participants;
            const tr = document.createElement('tr');
            tr.innerHTML = `
                <td><code>${room.id}</code></td>
                <td>${room.created}</td>
                <td>${room.participants}</td>
                <td><span class="status-badge">${room.status}</span></td>
                <td>
                    <button class="action-btn" onclick="window.open('/index.html?room=${room.id}', '_blank')">View</button>
                    <button class="action-btn delete" onclick="closeRoom('${room.id}')">Close</button>
                </td>
            `;
            roomTableBody.appendChild(tr);
        });

        totalRoomsEl.textContent = rooms.length;
        totalParticipantsEl.textContent = totalParts;
    }

    window.closeRoom = async (roomId) => {
        if (!confirm(`Are you sure you want to close room ${roomId} and clear all participant logs?`)) return;
        try {
            await fetch(`/api/close-room/${roomId}`, { method: 'POST' });
            fetchRooms(); // Refresh
        } catch (e) {
            console.error('Failed to close room:', e);
        }
    };

    // Poll for updates
    fetchRooms();
    setInterval(fetchRooms, 5000);
});
