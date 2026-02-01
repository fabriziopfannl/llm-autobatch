use pyo3::prelude::*;
use std::sync::atomic::{AtomicU64, Ordering};

#[derive(Default)]
pub struct Metrics {
    pub total_batches: AtomicU64,
    pub total_items: AtomicU64,
    pub flush_max_batch: AtomicU64,
    pub flush_deadline: AtomicU64,
    pub flush_manual: AtomicU64,
}

impl Metrics {
    pub fn snapshot(&self) -> MetricsSnapshot {
        MetricsSnapshot {
            total_batches: self.total_batches.load(Ordering::Relaxed),
            total_items: self.total_items.load(Ordering::Relaxed),
            flush_max_batch: self.flush_max_batch.load(Ordering::Relaxed),
            flush_deadline: self.flush_deadline.load(Ordering::Relaxed),
            flush_manual: self.flush_manual.load(Ordering::Relaxed),
        }
    }
}

#[pyclass]
#[derive(Clone)]
pub struct MetricsSnapshot {
    #[pyo3(get)]
    pub total_batches: u64,
    #[pyo3(get)]
    pub total_items: u64,
    #[pyo3(get)]
    pub flush_max_batch: u64,
    #[pyo3(get)]
    pub flush_deadline: u64,
    #[pyo3(get)]
    pub flush_manual: u64,
}

#[pymethods]
impl MetricsSnapshot {
    pub fn as_dict(&self, py: Python) -> PyResult<PyObject> {
        let d = pyo3::types::PyDict::new_bound(py);
        d.set_item("total_batches", self.total_batches)?;
        d.set_item("total_items", self.total_items)?;
        d.set_item("flush_max_batch", self.flush_max_batch)?;
        d.set_item("flush_deadline", self.flush_deadline)?;
        d.set_item("flush_manual", self.flush_manual)?;
        Ok(d.into())
    }
}
