use crate::types::ConnID;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Our global unique connection id counter.
static NEXT_CONNECTION_ID: AtomicUsize = AtomicUsize::new(1);

pub fn new_conn_id() -> ConnID {
    NEXT_CONNECTION_ID.fetch_add(1, Ordering::Relaxed)
}
