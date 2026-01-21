#!/bin/bash
# Helper script to run code coverage with correct exclusions

echo "Running cargo-tarpaulin with browser-specific exclusions..."

cargo tarpaulin --out Html --workspace \
  --exclude-files \
    "crates/wasm/**/*" \
    "crates/media/src/peer_connection.rs" \
    "crates/media/src/data_channel.rs" \
    "crates/media/src/ice_handler.rs" \
    "crates/media/src/stream_manager.rs" \
  $@

echo ""
echo "Report generated: tarpaulin-report.html"
