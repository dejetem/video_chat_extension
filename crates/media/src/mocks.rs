// Since we can't easily mock web-sys types directly in unit tests running on host,
// we often verify logic that DOESN'T depend on them, or we use traits.
// For Stage 3, we'll focus on mocking internal logic if we refactor to traits.
// Currently, `PeerConnectionManager` uses struct-impls, so we'll leave this empty
// or use it for higher-level mocks in the future.

// Placeholder for future mocks
pub struct MockWebRtc;
