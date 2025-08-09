use prometheus_client::{
    encoding::EncodeLabelSet,
    metrics::{family::Family, gauge::Gauge},
    registry::Registry,
};
use std::sync::atomic::AtomicU64;

#[derive(Clone, Debug, Hash, PartialEq, Eq, EncodeLabelSet)]
pub struct Labels {
    pub target: String,
}

pub struct PrometheusState {
    pub registry: Registry,
    pub power_metric: Family<Labels, Gauge<f64, AtomicU64>>,
    pub uptime_metric: Family<Labels, Gauge>,
    pub relay_ison_metric: Family<Labels, Gauge>,
    pub is_valid_metric: Family<Labels, Gauge>,
    pub total_power_metric: Family<Labels, Gauge<f64, AtomicU64>>,
}

pub fn init_prometheus_sate() -> PrometheusState {
    let mut prometheus_state = PrometheusState {
        registry: <Registry>::default(),
        power_metric: Family::<Labels, Gauge<f64, AtomicU64>>::default(),
        uptime_metric: Family::<Labels, Gauge>::default(),
        relay_ison_metric: Family::<Labels, Gauge>::default(),
        is_valid_metric: Family::<Labels, Gauge>::default(),
        total_power_metric: Family::<Labels, Gauge<f64, AtomicU64>>::default(),
    };

    prometheus_state.registry.register(
        "shelly_power_w",
        "The power value in this instant.",
        prometheus_state.power_metric.clone(),
    );
    prometheus_state.registry.register(
        "shelly_uptime",
        "Time the device has been running in seconds",
        prometheus_state.uptime_metric.clone(),
    );
    prometheus_state.registry.register(
        "shelly_relay_ison",
        "Is the relay on or off",
        prometheus_state.relay_ison_metric.clone(),
    );
    prometheus_state.registry.register(
        "shelly_power_is_valid",
        "Whether power metering self-checks OK",
        prometheus_state.is_valid_metric.clone(),
    );
    prometheus_state.registry.register(
        "shelly_power_total_w",
        "Total power consumption in watts since last power cycle",
        prometheus_state.total_power_metric.clone(),
    );

    return prometheus_state;
}
