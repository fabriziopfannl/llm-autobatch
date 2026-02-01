use pyo3::Py;
use pyo3::PyAny;
use std::sync::mpsc::Sender;
use std::time::Instant;

#[derive(Clone, Copy)]
pub enum Backpressure {
    Block,
    Drop,
    Passthrough,
}

pub struct Pending {
    pub enqueued_at: Instant,
    pub item: Py<PyAny>,
    pub executor: Py<PyAny>,
    pub tx: Sender<Result<Py<PyAny>, String>>,
}
