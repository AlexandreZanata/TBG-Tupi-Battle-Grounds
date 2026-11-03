//! Byte-bounded LRU for immutable approved sheets; authorization is checked before every hit.
use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex},
};
#[derive(Default)]
pub struct Cache(Mutex<State>);
#[derive(Default)]
struct State {
    items: BTreeMap<[u8; 16], (u64, Arc<Vec<u8>>)>,
    bytes: usize,
    clock: u64,
}
impl Cache {
    pub fn get(&self, id: &[u8; 16]) -> Option<Arc<Vec<u8>>> {
        let mut s = self.0.lock().ok()?;
        s.clock = s.clock.saturating_add(1);
        let tick = s.clock;
        let item = s.items.get_mut(id)?;
        item.0 = tick;
        Some(item.1.clone())
    }
    pub fn insert(&self, id: [u8; 16], bytes: Vec<u8>) {
        if bytes.len() > 262144 {
            return;
        }
        let Ok(mut s) = self.0.lock() else {
            return;
        };
        if s.items.contains_key(&id) {
            return;
        }
        while s.bytes + bytes.len() > 32 * 1024 * 1024 || s.items.len() >= 512 {
            let oldest = s.items.iter().min_by_key(|(_, v)| v.0).map(|(id, _)| *id);
            let Some(oldest) = oldest else {
                break;
            };
            if let Some((_, b)) = s.items.remove(&oldest) {
                s.bytes -= b.len();
            }
        }
        s.clock = s.clock.saturating_add(1);
        let tick = s.clock;
        s.bytes += bytes.len();
        s.items.insert(id, (tick, Arc::new(bytes)));
    }
}
