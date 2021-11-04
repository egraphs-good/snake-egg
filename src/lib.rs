use once_cell::sync::Lazy;

use std::cmp::Ordering;
use std::sync::Mutex;
use std::{borrow::Cow, fmt::Display, hash::Hash, time::Duration};

use pyo3::AsPyPointer;
use pyo3::{
    basic::CompareOp,
    prelude::*,
    types::{PyList, PyString, PyTuple, PyType},
    PyObjectProtocol,
};

macro_rules! impl_py_object {
    ($t:ty) => {
        #[pyproto]
        impl PyObjectProtocol for $t {
            fn __str__(&self) -> String {
                self.0.to_string()
            }

            fn __repr__(&self) -> String {
                format!(concat!(stringify!($t), "({})"), self.0)
            }

            fn __richcmp__(&self, other: Self, op: CompareOp) -> bool {
                match op {
                    CompareOp::Lt => self.0 < other.0,
                    CompareOp::Le => self.0 <= other.0,
                    CompareOp::Eq => self.0 == other.0,
                    CompareOp::Ne => self.0 != other.0,
                    CompareOp::Gt => self.0 > other.0,
                    CompareOp::Ge => self.0 >= other.0,
                }
            }
        }
    };
}

#[pyclass]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Id(egg::Id);

impl_py_object!(Id);

#[pyclass]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Var(egg::Var);

impl_py_object!(Var);

impl Var {
    fn from_str(str: &str) -> Self {
        let v = format!("?{}", str);
        Var(v.parse().unwrap())
    }
}

#[pymethods]
impl Var {
    #[new]
    fn new(str: &PyString) -> Self {
        Self::from_str(str.to_string_lossy().as_ref())
    }
}

#[derive(Debug, Clone)]
struct PyLang {
    obj: PyObject,
    children: Vec<egg::Id>,
}

impl PyLang {
    fn op(ty: &PyType, children: impl IntoIterator<Item = egg::Id>) -> Self {
        let any = ty.as_ref();
        let py = any.py();
        Self {
            obj: any.to_object(py),
            children: children.into_iter().collect(),
        }
    }

    fn leaf(any: &PyAny) -> Self {
        struct Hashable {
            obj: PyObject,
            hash: isize,
        }

        impl Hash for Hashable {
            fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
                self.hash.hash(state);
            }
        }

        impl PartialEq for Hashable {
            fn eq(&self, other: &Self) -> bool {
                let py = unsafe { Python::assume_gil_acquired() };
                let cmp = self.obj.as_ref(py).rich_compare(&other.obj, CompareOp::Eq);
                cmp.unwrap().is_true().unwrap()
            }
        }

        impl Eq for Hashable {}

        static LEAVES: Lazy<Mutex<hashbrown::HashSet<Hashable>>> = Lazy::new(Default::default);

        let hash = any.hash().expect("failed to hash");
        let py = any.py();
        let obj = any.to_object(py);

        let mut leaves = LEAVES.lock().unwrap();
        let hashable = leaves.get_or_insert(Hashable { obj, hash });

        Self {
            obj: hashable.obj.clone(),
            children: vec![],
        }
    }
}

impl PartialEq for PyLang {
    fn eq(&self, other: &Self) -> bool {
        self.obj.as_ptr() == other.obj.as_ptr() && self.children == other.children
    }
}

impl Hash for PyLang {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.obj.as_ptr().hash(state);
        self.children.hash(state);
    }
}

impl Ord for PyLang {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("comparison failed")
    }
}

impl PartialOrd for PyLang {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.obj.as_ptr().partial_cmp(&other.obj.as_ptr()) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        self.children.partial_cmp(&other.children)
    }
}

impl Eq for PyLang {}

impl egg::Language for PyLang {
    fn matches(&self, other: &Self) -> bool {
        self.obj.as_ptr() == other.obj.as_ptr() && self.children.len() == other.children.len()
    }

    fn children(&self) -> &[egg::Id] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [egg::Id] {
        &mut self.children
    }
}

impl Display for PyLang {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Python::with_gil(|py| match self.obj.as_ref(py).str() {
            Ok(s) => s.fmt(f),
            Err(_) => "<<NODE>>".fmt(f),
        })
    }
}

#[pyclass]
struct Pattern {
    pattern: egg::Pattern<PyLang>,
}

#[pymethods]
impl Pattern {
    #[new]
    fn new(tree: &PyAny) -> Self {
        let mut ast = egg::PatternAst::default();
        build_pattern(&mut ast, tree);
        let pattern = egg::Pattern::from(ast);
        Self { pattern }
    }
}

fn build_pattern(ast: &mut egg::PatternAst<PyLang>, tree: &PyAny) -> egg::Id {
    if let Ok(id) = tree.extract::<Id>() {
        panic!("Ids are unsupported in patterns: {}", id.0)
    } else if let Ok(var) = tree.extract::<Var>() {
        ast.add(egg::ENodeOrVar::Var(var.0))
    } else if let Ok(tuple) = tree.downcast::<PyTuple>() {
        let op = PyLang::op(
            tree.get_type(),
            tuple.iter().map(|child| build_pattern(ast, child)),
        );
        ast.add(egg::ENodeOrVar::ENode(op))
    } else {
        ast.add(egg::ENodeOrVar::ENode(PyLang::leaf(tree)))
    }
}

#[pyclass]
struct Rewrite {
    rewrite: egg::Rewrite<PyLang, ()>,
}

#[pymethods]
impl Rewrite {
    #[new]
    #[args(name = "\"\"")]
    fn new(lhs: &PyAny, rhs: &PyAny, name: &str) -> Self {
        let searcher = Pattern::new(lhs).pattern;
        let applier = Pattern::new(rhs).pattern;

        let mut name = Cow::Borrowed(name);
        if name == "" {
            name = Cow::Owned(format!("{} => {}", searcher, applier));
        }
        let rewrite = egg::Rewrite::new(name, searcher, applier).expect("Failed to create rewrite");
        Rewrite { rewrite }
    }

    #[getter]
    fn name(&self) -> &str {
        self.rewrite.name.as_str()
    }
}

#[pyclass]
#[derive(Default)]
pub struct EGraph {
    egraph: egg::EGraph<PyLang, ()>,
}

type Runner = egg::Runner<PyLang, (), ()>;

#[pymethods]
impl EGraph {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    fn add(&mut self, expr: &PyAny) -> Id {
        Id(self.add_rec(expr))
    }

    #[args(exprs = "*")]
    fn union(&mut self, exprs: &PyTuple) -> bool {
        assert!(exprs.len() > 1);
        let mut exprs = exprs.iter();
        let id = self.add(exprs.next().unwrap()).0;
        let mut did_something = false;
        for expr in exprs {
            let added = self.add(expr);
            did_something |= self.egraph.union(id, added.0);
        }
        did_something
    }

    #[args(exprs = "*")]
    fn equiv(&mut self, exprs: &PyTuple) -> bool {
        assert!(exprs.len() > 1);
        let mut exprs = exprs.iter();
        let id = self.add(exprs.next().unwrap()).0;
        let mut all_equiv = true;
        for expr in exprs {
            let added = self.add(expr);
            all_equiv &= added.0 == id
        }
        all_equiv
    }

    fn rebuild(&mut self) -> usize {
        self.egraph.rebuild()
    }

    #[args(iters = "10", time_limit = "10.0", node_limit = "100_000")]
    fn run(
        &mut self,
        rewrites: &PyList,
        iter_limit: usize,
        time_limit: f64,
        node_limit: usize,
    ) -> PyResult<()> {
        let refs = rewrites
            .iter()
            .map(FromPyObject::extract)
            .collect::<PyResult<Vec<PyRef<Rewrite>>>>()?;
        let egraph = std::mem::take(&mut self.egraph);
        let runner = Runner::new(())
            .with_iter_limit(iter_limit)
            .with_node_limit(node_limit)
            .with_time_limit(Duration::from_secs_f64(time_limit))
            .with_egraph(egraph)
            .run(refs.iter().map(|r| &r.rewrite));

        self.egraph = runner.egraph;
        Ok(())
    }
}

impl EGraph {
    fn add_rec(&mut self, expr: &PyAny) -> egg::Id {
        if let Ok(Id(id)) = expr.extract() {
            self.egraph.find(id)
        } else if let Ok(Var(var)) = expr.extract() {
            panic!("Can't add a var: {}", var)
        } else if let Ok(tuple) = expr.downcast::<PyTuple>() {
            let enode = PyLang::op(
                expr.get_type(),
                tuple.iter().map(|child| self.add_rec(child)),
            );
            self.egraph.add(enode)
        } else {
            self.egraph.add(PyLang::leaf(expr))
        }
    }
}

fn singleton_or_tuple<T, TS>(py: Python<'_>, elems: TS) -> PyObject
where
    T: IntoPy<PyObject>,
    TS: IntoIterator<Item = T>,
    TS::IntoIter: ExactSizeIterator<Item = T>,
{
    let mut elems = elems.into_iter();
    if elems.len() == 1 {
        elems.next().unwrap().into_py(py)
    } else {
        PyTuple::new(py, elems.map(|x| x.into_py(py))).into_py(py)
    }
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn snake_egg(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<EGraph>()?;
    m.add_class::<Id>()?;
    m.add_class::<Var>()?;
    m.add_class::<Pattern>()?;
    m.add_class::<Rewrite>()?;

    #[pyfn(m)]
    fn vars(py: Python<'_>, vars: &PyString) -> PyObject {
        let s = vars.to_string_lossy();
        let strs: Vec<&str> = s.split_whitespace().collect();
        singleton_or_tuple(py, strs.iter().map(|s| Var::from_str(s)))
    }
    Ok(())
}
