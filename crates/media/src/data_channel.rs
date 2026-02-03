use crate::{MediaError, Result};
use log::{debug, info};
use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use wasm_bindgen::prelude::*;
use web_sys::{RtcDataChannel, RtcPeerConnection};

pub struct DataChannelManager {
    data_channel: Option<RtcDataChannel>,
    message_queue: Arc<Mutex<VecDeque<String>>>,
}

impl Default for DataChannelManager {
    fn default() -> Self {
        Self::new()
    }
}

impl DataChannelManager {
    pub fn new() -> Self {
        Self {
            data_channel: None,
            message_queue: Arc::new(Mutex::new(VecDeque::new())),
        }
    }

    pub fn create_data_channel(
        &mut self,
        connection: &RtcPeerConnection,
        label: &str,
        on_message_cb: Option<Box<dyn Fn(String) + Send + 'static>>,
    ) -> Result<()> {
        let channel = connection.create_data_channel(label);
        info!("Created DataChannel: {}", label);

        let queue = self.message_queue.clone();
        let channel_clone = channel.clone();
        let on_open = Closure::wrap(Box::new(move || {
            info!("DataChannel opened, flushing queue");
            let mut q = queue.lock().unwrap();
            while let Some(msg) = q.pop_front() {
                let _ = channel_clone.send_with_str(&msg);
            }
        }) as Box<dyn FnMut()>);
        channel.set_onopen(Some(on_open.as_ref().unchecked_ref()));
        on_open.forget();

        let on_message = Closure::wrap(Box::new(move |event: web_sys::MessageEvent| {
            if let Some(data) = event.data().as_string() {
                debug!("Received message: {}", data);
                if let Some(ref cb) = on_message_cb {
                    cb(data);
                }
            }
        }) as Box<dyn FnMut(_)>);
        channel.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
        on_message.forget();

        self.data_channel = Some(channel);
        Ok(())
    }

    pub fn send_message(&self, message: &str) -> Result<()> {
        if let Some(channel) = &self.data_channel {
            match channel.ready_state() {
                web_sys::RtcDataChannelState::Open => {
                    channel
                        .send_with_str(message)
                        .map_err(|e| MediaError::DataChannel(format!("Failed to send: {:?}", e)))?;
                    Ok(())
                }
                web_sys::RtcDataChannelState::Connecting => {
                    info!("DataChannel connecting, queuing message");
                    self.message_queue
                        .lock()
                        .unwrap()
                        .push_back(message.to_string());
                    Ok(())
                }
                _ => Err(MediaError::DataChannel(
                    "DataChannel is closed or closing".into(),
                )),
            }
        } else {
            Err(MediaError::DataChannel("No DataChannel initialized".into()))
        }
    }
}
