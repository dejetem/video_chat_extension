document.addEventListener('DOMContentLoaded', () => {
    console.log('Admin Dashboard Initialized');

    // Mock Data
    const mockRooms = [
        { id: 'room-101', created: '2026-01-22 10:30', participants: 4, status: 'Active' },
        { id: 'dev-session', created: '2026-01-22 11:45', participants: 2, status: 'Active' },
        { id: 'quick-chat', created: '2026-01-22 12:10', participants: 0, status: 'Idle' }
    ];

    const roomTableBody = document.getElementById('roomTableBody');
    const totalRoomsEl = document.getElementById('totalRooms');
    const totalParticipantsEl = document.getElementById('totalParticipants');

    function renderRooms() {
        roomTableBody.innerHTML = '';
        let totalParts = 0;

        mockRooms.forEach(room => {
            totalParts += room.participants;
            const tr = document.createElement('tr');
            tr.innerHTML = `
                <td><code>${room.id}</code></td>
                <td>${room.created}</td>
                <td>${room.participants}</td>
                <td><span class="status-badge">${room.status}</span></td>
                <td>
                    <button class="action-btn">View</button>
                    <button class="action-btn delete">Close</button>
                </td>
            `;
            roomTableBody.appendChild(tr);
        });

        totalRoomsEl.textContent = mockRooms.length;
        totalParticipantsEl.textContent = totalParts;
    }

    document.getElementById('createRoomBtn').addEventListener('click', () => {
        const id = 'room-' + Math.floor(Math.random() * 1000);
        mockRooms.push({
            id: id,
            created: new Date().toISOString().slice(0, 16).replace('T', ' '),
            participants: 0,
            status: 'Idle'
        });
        renderRooms();
    });

    renderRooms();
});
