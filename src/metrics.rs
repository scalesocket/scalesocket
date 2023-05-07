use crate::types::RoomID;
use {
    prometheus_client::encoding::EncodeLabelSet,
    prometheus_client::metrics::{counter::Counter, family::Family, gauge::Gauge},
    prometheus_client::registry::Registry,
};

#[derive(Clone, Hash, PartialEq, Eq, EncodeLabelSet, Debug)]
pub struct Labels {
    room: RoomID,
}

#[derive(Clone)]
pub struct Metrics {
    pub ws_connections_counter: Family<Labels, Counter>,
    pub ws_connections_open_gauge: Family<Labels, Gauge>,
}

impl Metrics {
    pub fn new(registry: &mut Option<Registry>) -> Self {
        let ws_connections_counter = Family::<Labels, Counter>::default();
        let ws_connections_open_gauge = Family::<Labels, Gauge>::default();

        if let Some(registry) = registry {
            registry.register(
                "scalesocket_websocket_connections_total",
                "Number of total websocket connections",
                ws_connections_counter.clone(),
            );
            registry.register(
                "scalesocket_websocket_connections_open",
                "number of open websocket connections",
                ws_connections_open_gauge.clone(),
            );
        }

        Self {
            ws_connections_counter,
            ws_connections_open_gauge,
        }
    }

    pub fn inc_ws_connections(&self, room: &str) {
        self.ws_connections_counter
            .get_or_create(&Labels {
                room: room.to_string(),
            })
            .inc();
        self.ws_connections_open_gauge
            .get_or_create(&Labels {
                room: room.to_string(),
            })
            .inc();
    }

    pub fn dec_ws_connections(&self, room: &str) {
        self.ws_connections_open_gauge
            .get_or_create(&Labels {
                room: room.to_string(),
            })
            .dec();
    }

    pub fn get_room(&self, room: RoomID) -> serde_json::Value {
        serde_json::json!({ "connections": self.get_room_connections(room) })
    }

    pub fn get_room_connections(&self, room: RoomID) -> i64 {
        self.ws_connections_open_gauge
            .get_or_create(&Labels { room })
            .get()
    }
}
