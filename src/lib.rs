mod core;
mod lang;
mod util;

use crate::core::*;
use crate::lang::*;

use pyo3::{prelude::*, types::PyString};

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn _internal(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<PyEGraph>()?;
    m.add_class::<PyId>()?;
    m.add_class::<PyVar>()?;
    m.add_class::<PyPattern>()?;
    m.add_class::<PyRewrite>()?;

    #[pyfn(m)]
    fn vars(vars: &PyString) -> Vec<PyVar> {
        let s = vars.to_string_lossy();
        s.split_whitespace().map(|s| PyVar::from_str(s)).collect()
    }
    Ok(())
}
