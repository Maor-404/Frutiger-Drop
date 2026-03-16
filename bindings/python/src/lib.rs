use pyo3::exceptions::PyValueError;
use pyo3::prelude::*;
use pyo3::types::PyBytes;

#[pyfunction]
fn apply_blur(py: Python<'_>, input: &Bound<'_, PyBytes>, width: u32, height: u32) -> PyResult<Py<PyBytes>> {
    let in_bytes = input.as_bytes();
    let out = frutiger_drop_core::apply_blur(in_bytes, width, height);
    Ok(PyBytes::new_bound(py, &out).into())
}

#[pyfunction]
fn apply_tint(py: Python<'_>, rgba: &Bound<'_, PyBytes>, tint: (u8, u8, u8, u8)) -> PyResult<Py<PyBytes>> {
    let out = frutiger_drop_core::apply_tint(rgba.as_bytes(), tint);
    Ok(PyBytes::new_bound(py, &out).into())
}

#[pyfunction]
fn composite_layers(
    py: Python<'_>,
    bottom_rgba: &Bound<'_, PyBytes>,
    top_rgba: &Bound<'_, PyBytes>,
) -> PyResult<Py<PyBytes>> {
    let b = bottom_rgba.as_bytes();
    let t = top_rgba.as_bytes();
    if b.len() != t.len() {
        return Err(PyValueError::new_err("Layer buffers must match in length"));
    }
    let out = frutiger_drop_core::composite_layers(b, t);
    Ok(PyBytes::new_bound(py, &out).into())
}

#[pymodule]
fn _native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(apply_blur, m)?)?;
    m.add_function(wrap_pyfunction!(apply_tint, m)?)?;
    m.add_function(wrap_pyfunction!(composite_layers, m)?)?;
    Ok(())
}
