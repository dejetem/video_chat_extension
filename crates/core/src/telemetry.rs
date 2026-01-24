use tracing::{error, info, warn};

pub struct Telemetry;

impl Telemetry {
    pub fn log_event(event_name: &str, properties: Vec<(&str, &str)>) {
        info!("Telemetry Event: {} - {:?}", event_name, properties);
    }

    pub fn log_error(error_msg: &str) {
        error!("Telemetry Error: {}", error_msg);
    }

    pub fn log_performance(metric: &str, value_ms: u64) {
        info!("Performance Metric: {} = {}ms", metric, value_ms);
    }

    pub fn track_metric(name: &str, value: f64) {
        info!("Metric: {} = {}", name, value);
    }

    pub fn track_error(code: &str, message: &str) {
        error!("Error [{}]: {}", code, message);
    }

    pub fn capture_analytics(event: &str) {
        // Privacy-respecting analytics: No PII, just event name
        info!("Analytics Event: {}", event);
    }
}
