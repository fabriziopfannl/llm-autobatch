use pyo3::prelude::*;

mod core;
mod metrics;
mod queue;

#[pymodule]
fn _native(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<core::BatcherCore>()?;
    m.add_class::<metrics::MetricsSnapshot>()?;
    Ok(())
}
