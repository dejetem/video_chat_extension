use crate::{MediaError, Result};
use web_sys::{MediaStream, RtcPeerConnection};

pub struct StreamManager {
    local_stream: Option<MediaStream>,
    remote_streams: Vec<MediaStream>,
}

impl StreamManager {
    pub fn new() -> Self {
        Self {
            local_stream: None,
            remote_streams: Vec::new(),
        }
    }

    pub fn set_local_stream(&mut self, stream: MediaStream) {
        self.local_stream = Some(stream);
    }

    pub fn get_local_stream(&self) -> Option<&MediaStream> {
        self.local_stream.as_ref()
    }

    pub fn add_track_to_connection(&self, connection: &RtcPeerConnection) -> Result<()> {
        if let Some(stream) = &self.local_stream {
            let streams = js_sys::Array::new();
            streams.push(stream);

            for track in stream.get_tracks().iter() {
                let track = web_sys::MediaStreamTrack::from(track);
                let streams_arr = js_sys::Array::new();
                connection.add_track(&track, stream, &streams_arr);
            }
            Ok(())
        } else {
            Err(MediaError::Stream("No local stream to add".into()))
        }
    }

    pub fn add_remote_stream(&mut self, stream: MediaStream) {
        self.remote_streams.push(stream);
    }
}
