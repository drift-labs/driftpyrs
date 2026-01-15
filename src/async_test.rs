use pyo3::prelude::*;

/// A simple async function that sleeps for the specified number of seconds
/// and returns a message. This demonstrates the async bridge between Python
/// and Rust using pyo3-async-runtimes.
#[pyfunction]
pub fn sleep_and_return<'py>(py: Python<'py>, seconds: u64) -> PyResult<Bound<'py, PyAny>> {
    pyo3_async_runtimes::tokio::future_into_py(py, async move {
        #[cfg(feature = "observability")]
        {
            let t = std::thread::current();
            tracing::info!(
                phase = "start",
                seconds,
                thread_id = ?t.id(),
                thread_name = ?t.name()
            );
        }

        tokio::time::sleep(tokio::time::Duration::from_secs(seconds)).await;

        #[cfg(feature = "observability")]
        {
            let t = std::thread::current();
            tracing::info!(
                phase = "done",
                seconds,
                thread_id = ?t.id(),
                thread_name = ?t.name()
            );
        }

        Ok(format!("Slept for {} seconds", seconds))
    })
}
