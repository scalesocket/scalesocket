use {
    prometheus_client::encoding::EncodeLabelSet,
    prometheus_client::metrics::{counter::Counter, family::Family, gauge::Gauge},
    prometheus_client::registry::Registry,
    serde_json::{self, json, Value},
    std::collections::{HashMap, HashSet},
    std::sync::{Arc, RwLock},
};

use crate::types::RoomID;

#[derive(Clone, Hash, PartialEq, Eq, EncodeLabelSet, Debug)]
pub struct Labels {
    room: RoomID,
}

#[derive(Clone)]
pub struct Metrics {
    metas: Arc<RwLock<HashMap<String, Value>>>,
    ws_connections_counter: Family<Labels, Counter>,
    ws_connections_open_gauge: Family<Labels, Gauge>,
    // prometheus_client does not expose iterators over `Metrics` or `Labels`
    // https://github.com/prometheus/client_rust/issues/131
    ws_connections_labels: Option<Arc<RwLock<HashSet<String>>>>,
}

impl Metrics {
    pub fn new(registry: &mut Option<Registry>, track_labels: bool) -> Self {
        let ws_connections_counter = Family::<Labels, Counter>::default();
        let ws_connections_open_gauge = Family::<Labels, Gauge>::default();
        let ws_connections_labels =
            track_labels.then(|| Arc::new(RwLock::new(HashSet::with_capacity(100))));

        if let Some(registry) = registry {
            registry.register(
                "scalesocket_websocket_connections",
                "Number of total websocket connections",
                ws_connections_counter.clone(),
            );
            registry.register(
                "scalesocket_websocket_connections_open",
                "Number of open websocket connections",
                ws_connections_open_gauge.clone(),
            );
        }

        Self {
            metas: Arc::new(RwLock::new(HashMap::new())),
            ws_connections_counter,
            ws_connections_open_gauge,
            ws_connections_labels,
        }
    }

    pub fn inc_ws_connections(&self, room: &str) {
        self.ws_connections_counter
            .get_or_create(&Labels {
                room: room.to_string(),
            })
            .inc();
        let add_label = self
            .ws_connections_open_gauge
            .get_or_create(&Labels {
                room: room.to_string(),
            })
            .inc()
            == 0;

        if add_label {
            if let Some(rooms) = &self.ws_connections_labels {
                rooms
                    .write()
                    .expect("poisoned lock")
                    .insert(room.to_owned());
            }
        }
    }

    pub fn dec_ws_connections(&self, room: &str) {
        let remove_label = self
            .ws_connections_open_gauge
            .get_or_create(&Labels {
                room: room.to_string(),
            })
            .dec()
            == 1;

        if remove_label {
            if let Some(rooms) = &self.ws_connections_labels {
                rooms
                    .write()
                    .expect("poisoned lock")
                    .insert(room.to_owned());
            }
        }
    }

    pub fn set_metadata(&self, room: &str, metadata: Value) {
        self.metas
            .write()
            .expect("poisoned lock")
            .insert(room.to_owned(), metadata);
    }

    pub fn clear(&self, room: &str) {
        self.ws_connections_open_gauge.remove(&Labels {
            room: room.to_owned(),
        });

        if let Some(rooms) = &self.ws_connections_labels {
            rooms.write().expect("poisoned lock").remove(room);
        }
    }

    pub fn get_rooms(&self) -> Vec<Value> {
        match &self.ws_connections_labels {
            Some(rooms) => rooms
                .read()
                .expect("poisoned lock")
                .iter()
                .map(|room| self.get_room(room.clone()))
                .collect::<Vec<_>>(),
            None => vec![],
        }
    }

    pub fn get_room(&self, room: RoomID) -> Value {
        json!({
           "name": room.clone(),
           "connections": self.get_room_connections(room.clone()),
           "meta": self.metas.read().expect("poisoned lock").get(&room).cloned()
        })
    }

    pub fn get_room_connections(&self, room: RoomID) -> i64 {
        self.ws_connections_open_gauge
            .get_or_create(&Labels { room })
            .get()
    }
}
