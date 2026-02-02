use crate::metrics::{Metrics, MetricsSnapshot};
use crate::queue::{Backpressure, Pending};
use parking_lot::{Condvar, Mutex};
use pyo3::prelude::*;
use pyo3::types::PyList;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

#[pyclass]
pub struct BatcherCore {
    max_batch: usize,
    max_wait_ms: u64,
    backpressure: Backpressure,
    queue: Arc<Mutex<Vec<Pending>>>,
    cv: Arc<Condvar>,
    metrics: Arc<Metrics>,
    stop: Arc<AtomicBool>,
}

#[pymethods]
impl BatcherCore {
    #[new]
    pub fn new(max_batch: usize, max_wait_ms: u64, backpressure: &str) -> PyResult<Self> {
        let bp = match backpressure {
            "block" => Backpressure::Block,
            "drop" => Backpressure::Drop,
            "passthrough" => Backpressure::Passthrough,
            _ => {
                return Err(pyo3::exceptions::PyValueError::new_err(
                    "backpressure must be one of: block, drop, passthrough",
                ))
            }
        };

        let core = BatcherCore {
            max_batch,
            max_wait_ms,
            backpressure: bp,
            queue: Arc::new(Mutex::new(Vec::with_capacity(max_batch))),
            cv: Arc::new(Condvar::new()),
            metrics: Arc::new(Metrics::default()),
            stop: Arc::new(AtomicBool::new(false)),
        };

        core.spawn_worker();

        Ok(core)
    }

    pub fn submit(&self, py: Python, item: PyObject, executor: PyObject) -> PyResult<PyObject> {
        let (tx, rx) = mpsc::channel();

        let pending = Pending {
            enqueued_at: Instant::now(),
            item,
            executor,
            tx,
        };

        {
            let q = self.queue.lock();
            if q.len() >= self.max_batch {
                match self.backpressure {
                    Backpressure::Drop => {
                        return Err(pyo3::exceptions::PyRuntimeError::new_err(
                            "queue full (backpressure=drop)",
                        ));
                    }
                    Backpressure::Passthrough => {
                        drop(q);
                        return self.call_executor_direct(py, pending);
                    }
                    Backpressure::Block => {}
                }
            }
        }

        let recv_result = py.allow_threads(move || {
            {
                let mut q = self.queue.lock();
                while q.len() >= self.max_batch {
                    self.cv.wait(&mut q);
                }
                q.push(pending);
            }
            self.cv.notify_one();
            rx.recv()
        });

        match recv_result {
            Ok(Ok(value)) => Ok(value.into_py(py)),
            Ok(Err(msg)) => Err(pyo3::exceptions::PyRuntimeError::new_err(msg)),
            Err(_) => Err(pyo3::exceptions::PyRuntimeError::new_err(
                "batcher worker stopped",
            )),
        }
    }

    pub fn flush(&self) {
        self.metrics.flush_manual.fetch_add(1, Ordering::Relaxed);
        self.cv.notify_all();
    }

    pub fn metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    pub fn close(&self) {
        self.stop.store(true, Ordering::Relaxed);
        self.cv.notify_all();
    }
}

impl BatcherCore {
    fn spawn_worker(&self) {
        let queue = Arc::clone(&self.queue);
        let cv = Arc::clone(&self.cv);
        let metrics = Arc::clone(&self.metrics);
        let stop = Arc::clone(&self.stop);
        let max_batch = self.max_batch;
        let max_wait_ms = self.max_wait_ms;

        thread::spawn(move || loop {
            let mut batch: Vec<Pending> = Vec::new();

            {
                let mut q = queue.lock();
                while q.is_empty() && !stop.load(Ordering::Relaxed) {
                    cv.wait(&mut q);
                }

                if stop.load(Ordering::Relaxed) {
                    break;
                }

                let deadline = q[0].enqueued_at + Duration::from_millis(max_wait_ms);

                while q.len() < max_batch && Instant::now() < deadline {
                    let timeout = deadline.saturating_duration_since(Instant::now());
                    if timeout.is_zero() {
                        break;
                    }
                    cv.wait_for(&mut q, timeout);
                    if q.is_empty() {
                        break;
                    }
                }

                let take = std::cmp::min(q.len(), max_batch);
                batch.extend(q.drain(0..take));
                cv.notify_all();
            }

            if batch.is_empty() {
                continue;
            }

            if batch.len() >= max_batch {
                metrics.flush_max_batch.fetch_add(1, Ordering::Relaxed);
            } else {
                metrics.flush_deadline.fetch_add(1, Ordering::Relaxed);
            }

            metrics.total_batches.fetch_add(1, Ordering::Relaxed);
            metrics
                .total_items
                .fetch_add(batch.len() as u64, Ordering::Relaxed);

            Python::with_gil(|py| {
                let items = batch
                    .iter()
                    .map(|p| p.item.clone_ref(py).into_py(py))
                    .collect::<Vec<PyObject>>();

                let list = PyList::new_bound(py, &items);
                let exec = batch[0].executor.clone_ref(py);

                let result = exec.call1(py, (list.as_any(),));
                match result {
                    Ok(out) => {
                        let out_list = out.downcast_bound::<PyList>(py);
                        if let Ok(out_list) = out_list {
                            if out_list.len() != batch.len() {
                                let msg = format!(
                                    "executor returned {} items for {} inputs",
                                    out_list.len(),
                                    batch.len()
                                );
                                for p in batch {
                                    let _ = p.tx.send(Err(msg.clone()));
                                }
                                return;
                            }

                            for (i, p) in batch.into_iter().enumerate() {
                                let value = out_list.get_item(i).unwrap().into_py(py);
                                let _ = p.tx.send(Ok(value));
                            }
                        } else {
                            let msg = "executor must return a list".to_string();
                            for p in batch {
                                let _ = p.tx.send(Err(msg.clone()));
                            }
                        }
                    }
                    Err(err) => {
                        let msg = format!("executor error: {err}");
                        for p in batch {
                            let _ = p.tx.send(Err(msg.clone()));
                        }
                    }
                }
            });
        });
    }

    fn call_executor_direct(&self, py: Python, pending: Pending) -> PyResult<PyObject> {
        let list = PyList::new_bound(py, &[pending.item.clone_ref(py).into_py(py)]);
        let exec = pending.executor.clone_ref(py);
        let out = exec.call1(py, (list.as_any(),))?;
        let out_list = out.downcast_bound::<PyList>(py)?;
        if out_list.len() != 1 {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "executor must return one item in passthrough mode",
            ));
        }
        Ok(out_list.get_item(0).unwrap().into_py(py))
    }
}
    pub fn metrics(&self) -> MetricsSnapshot {
        self.metrics.snapshot()
    }

    pub fn close(&self) {
        self.stop.store(true, Ordering::Relaxed);
        self.cv.notify_all();
    }
}

impl BatcherCore {
    fn spawn_worker(&self) {
        let queue = Arc::clone(&self.queue);
        let cv = Arc::clone(&self.cv);
        let metrics = Arc::clone(&self.metrics);
        let stop = Arc::clone(&self.stop);
        let max_batch = self.max_batch;
        let max_wait_ms = self.max_wait_ms;

        thread::spawn(move || loop {
            let mut batch: Vec<Pending> = Vec::new();

            {
                let mut q = queue.lock();
                while q.is_empty() && !stop.load(Ordering::Relaxed) {
                    cv.wait(&mut q);
                }

                if stop.load(Ordering::Relaxed) {
                    break;
                }

                let deadline = q[0].enqueued_at + Duration::from_millis(max_wait_ms);

                while q.len() < max_batch && Instant::now() < deadline {
                    let timeout = deadline.saturating_duration_since(Instant::now());
                    if timeout.is_zero() {
                        break;
                    }
                    cv.wait_for(&mut q, timeout);
                    if q.is_empty() {
                        break;
                    }
                }

                let take = std::cmp::min(q.len(), max_batch);
                batch.extend(q.drain(0..take));
                cv.notify_all();
            }

            if batch.is_empty() {
                continue;
            }

            if batch.len() >= max_batch {
                metrics.flush_max_batch.fetch_add(1, Ordering::Relaxed);
            } else {
                metrics.flush_deadline.fetch_add(1, Ordering::Relaxed);
            }

            metrics.total_batches.fetch_add(1, Ordering::Relaxed);
            metrics
                .total_items
                .fetch_add(batch.len() as u64, Ordering::Relaxed);

            Python::with_gil(|py| {
                let items = batch
                    .iter()
                    .map(|p| p.item.clone_ref(py).into_py(py))
                    .collect::<Vec<PyObject>>();

                let list = PyList::new_bound(py, &items);
                let exec = batch[0].executor.clone_ref(py);

                let result = exec.call1(py, (list.as_any(),));
                match result {
                    Ok(out) => {
                        let out_list = out.downcast_bound::<PyList>(py);
                        if let Ok(out_list) = out_list {
                            if out_list.len() != batch.len() {
                                let msg = format!(
                                    "executor returned {} items for {} inputs",
                                    out_list.len(),
                                    batch.len()
                                );
                                for p in batch {
                                    let _ = p.tx.send(Err(msg.clone()));
                                }
                                return;
                            }

                            for (i, p) in batch.into_iter().enumerate() {
                                let value = out_list.get_item(i).unwrap().into_py(py);
                                let _ = p.tx.send(Ok(value));
                            }
                        } else {
                            let msg = "executor must return a list".to_string();
                            for p in batch {
                                let _ = p.tx.send(Err(msg.clone()));
                            }
                        }
                    }
                    Err(err) => {
                        let msg = format!("executor error: {err}");
                        for p in batch {
                            let _ = p.tx.send(Err(msg.clone()));
                        }
                    }
                }
            });
        });
    }

    fn call_executor_direct(&self, py: Python, pending: Pending) -> PyResult<PyObject> {
        let list = PyList::new_bound(py, &[pending.item.clone_ref(py).into_py(py)]);
        let exec = pending.executor.clone_ref(py);
        let out = exec.call1(py, (list.as_any(),))?;
        let out_list = out.downcast_bound::<PyList>(py)?;
        if out_list.len() != 1 {
            return Err(pyo3::exceptions::PyRuntimeError::new_err(
                "executor must return one item in passthrough mode",
            ));
        }
        Ok(out_list.get_item(0).unwrap().into_py(py))
    }
}
