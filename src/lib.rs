use std::{array::IntoIter, collections::HashMap, hash::Hash, time::Duration};

use pyo3::{
    basic::CompareOp,
    prelude::*,
    types::{PyList, PyString, PyTuple},
    AsPyPointer, PyNativeType, PyObjectProtocol, ToPyObject,
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
        let ty = tree.get_type_ptr() as usize;
        let ids: Vec<egg::Id> = tuple
            .iter()
            .map(|child| build_pattern(ast, child))
            .collect();
        ast.add(egg::ENodeOrVar::ENode(PyLang::Node(ty, ids)))
    } else if let Ok(n) = tree.extract::<i64>() {
        ast.add(egg::ENodeOrVar::ENode(PyLang::Int(n)))
    } else {
        let repr = tree.repr().expect("failed to repr");
        panic!("Cannot convert to pattern: {}", repr)
    }
}

#[pyclass]
struct Rewrite {
    rewrite: egg::Rewrite<PyLang, ()>,
}

#[pymethods]
impl Rewrite {
    #[new]
    fn new(from: &PyAny, to: &PyAny) -> Self {
        let searcher = Pattern::new(from).pattern;
        let applier = Pattern::new(to).pattern;
        let rewrite =
            egg::Rewrite::new("name", searcher, applier).expect("Failed to create rewrite");
        Rewrite { rewrite }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum PyLang {
    Int(i64),
    Node(usize, Vec<egg::Id>),
}

impl egg::Language for PyLang {
    fn matches(&self, other: &Self) -> bool {
        use PyLang::*;
        match (self, other) {
            (Node(op1, args1), Node(op2, args2)) => op1 == op2 && args1.len() == args2.len(),
            (Int(a), Int(b)) => a == b,
            _ => false,
        }
    }

    fn children(&self) -> &[egg::Id] {
        match self {
            PyLang::Node(_, args) => args,
            _ => &[],
        }
    }

    fn children_mut(&mut self) -> &mut [egg::Id] {
        match self {
            PyLang::Node(_, args) => args,
            _ => &mut [],
        }
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
        if let Ok(id) = expr.extract::<Id>() {
            self.egraph.find(id.0)
        } else if let Ok(tuple) = expr.downcast::<PyTuple>() {
            let ty = expr.get_type_ptr() as usize;
            let ids: Vec<egg::Id> = tuple.iter().map(|child| self.add_rec(child)).collect();
            self.egraph.add(PyLang::Node(ty, ids))
        } else if let Ok(n) = expr.extract::<i64>() {
            self.egraph.add(PyLang::Int(n))
        // } else if let Ok(s) = expr.extract::<&str>() {
        //     self.egraph.add(ENode::Symbol(s.into()))
        } else {
            let repr = expr.repr().expect("failed to repr");
            panic!("Cannot add {}", repr)
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
