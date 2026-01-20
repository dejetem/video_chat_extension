use crate::{MediaError, Result};
use log::{debug, info};
use wasm_bindgen::prelude::*;
use web_sys::{RtcDataChannel, RtcPeerConnection};

pub struct DataChannelManager {
    data_channel: Option<RtcDataChannel>,
}

impl DataChannelManager {
    pub fn new() -> Self {
        Self { data_channel: None }
    }

    pub fn create_data_channel(
        &mut self,
        connection: &RtcPeerConnection,
        label: &str,
    ) -> Result<()> {
        let channel = connection.create_data_channel(label);
        info!("Created DataChannel: {}", label);

        let on_open = Closure::wrap(Box::new(move || {
            info!("DataChannel opened");
        }) as Box<dyn FnMut()>);
        channel.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget(); // Leak to keep callback alive

        let on_message = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            if let Some(data) = event.data().as_string() {
                debug!("Received message: {}", data);
                // Here we would dispatch to a handler
            }
        }) as Box<dyn FnMut(_)>);
        channel.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();

        self.data_channel = Some(channel);
        Ok(())
    }

    pub fn send_message(&self, message: &str) -> Result<()> {
        if let Some(channel) = &self.data_channel {
            if channel.ready_state() == web_sys::RtcDataChannelState::Open {
                channel
                    .send_with_str(message)
                    .map_err(|e| MediaError::DataChannel(format!("Failed to send: {:?}", e)))?;
                Ok(())
            } else {
                Err(MediaError::DataChannel("DataChannel is not open".into()))
            }
        } else {
            Err(MediaError::DataChannel("No DataChannel initialized".into()))
        }
    }
}
