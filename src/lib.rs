use egg::{Language, RecExpr};
use once_cell::sync::Lazy;

use std::cmp::Ordering;
use std::sync::Mutex;
use std::{fmt::Display, hash::Hash, time::Duration};

use pyo3::AsPyPointer;
use pyo3::{
    basic::CompareOp,
    prelude::*,
    types::{PyList, PyString, PyTuple, PyType},
};

fn py_eq(a: &PyAny, b: impl ToPyObject) -> bool {
    a.rich_compare(b, CompareOp::Eq)
        .expect("Failed to compare")
        .is_true()
        .expect("Failed to extract bool")
}

macro_rules! py_object {
    (impl $t:ty { $($rest:tt)* }) => {
        #[pymethods]
        impl $t {
            $($rest)*

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
struct PyId(egg::Id);

py_object!(impl PyId {});

#[pyclass]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct PyVar(egg::Var);

py_object!(impl PyVar {
    #[new]
    fn new(str: &PyString) -> Self {
        Self::from_str(str.to_string_lossy().as_ref())
    }
});

impl PyVar {
    fn from_str(str: &str) -> Self {
        let v = format!("?{}", str);
        PyVar(v.parse().unwrap())
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
                py_eq(self.obj.as_ref(py), &other.obj)
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

    fn to_object<T: IntoPy<PyObject>>(&self, py: Python, f: impl FnMut(egg::Id) -> T) -> PyObject {
        if self.is_leaf() {
            self.obj.clone()
        } else {
            let children = self.children.iter().copied().map(f);
            let args = PyTuple::new(py, children.map(|o| o.into_py(py)));
            self.obj.call1(py, args).expect("Failed to construct")
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
struct PyPattern {
    pattern: egg::Pattern<PyLang>,
}

#[pymethods]
impl PyPattern {
    #[new]
    fn new(tree: &PyAny) -> Self {
        let mut ast = egg::PatternAst::default();
        build_pattern(&mut ast, tree);
        let pattern = egg::Pattern::from(ast);
        Self { pattern }
    }

    // // fn search(&self, egraph: &EGraph<L, N>) -> Vec<SearchMatches<L>> {
    // fn search(&self, egraph: &EGraph) -> Vec<SearchMatches> {
    //     let result = self.pattern.search(egraph);

    //     //.into_iter().map(|m| SearchMatches { matches: m }).collect()
    // }
}

fn build_pattern(ast: &mut egg::PatternAst<PyLang>, tree: &PyAny) -> egg::Id {
    if let Ok(id) = tree.extract::<PyId>() {
        panic!("Ids are unsupported in patterns: {}", id.0)
    } else if let Ok(var) = tree.extract::<PyVar>() {
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
struct PyRewrite {
    rewrite: egg::Rewrite<PyLang, PyAnalysis>,
}

#[pymethods]
impl PyRewrite {
    #[new]
    #[args(name = "\"\"")]
    fn new(lhs: &PyAny, rhs: &PyAny, name: &str) -> Self {
        let searcher = PyPattern::new(lhs).pattern;
        let applier = PyPattern::new(rhs).pattern;
        let rewrite = egg::Rewrite::new(name, searcher, applier).expect("Failed to create rewrite");
        PyRewrite { rewrite }
    }

    #[getter]
    fn name(&self) -> &str {
        self.rewrite.name.as_str()
    }
}

#[derive(Default)]
struct PyAnalysis {
    eval: Option<PyObject>,
}

impl egg::Analysis<PyLang> for PyAnalysis {
    type Data = Option<PyObject>;

    fn make(egraph: &egg::EGraph<PyLang, Self>, enode: &PyLang) -> Self::Data {
        let eval = egraph.analysis.eval.as_ref()?;
        let py = unsafe { Python::assume_gil_acquired() };

        // collect the children if they are not `None` in python
        let mut children = Vec::with_capacity(enode.len());
        for &id in enode.children() {
            let any = egraph[id].data.as_ref()?.as_ref(py);
            if any.is_none() {
                return None;
            } else {
                children.push(any)
            }
        }

        let res = eval
            .call1(py, (enode.obj.clone(), children))
            .expect("Failed to call eval");
        if res.is_none(py) {
            None
        } else {
            Some(res)
        }
    }

    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> egg::DidMerge {
        let py = unsafe { Python::assume_gil_acquired() };
        let aa = a.as_ref().map(|obj| obj.as_ref(py)).filter(|r| r.is_none());
        let bb = b.as_ref().map(|obj| obj.as_ref(py)).filter(|r| r.is_none());
        match (aa, bb) {
            (None, None) => egg::DidMerge(false, false),
            (None, Some(bb)) => {
                *a = Some(bb.to_object(py));
                egg::DidMerge(true, false)
            }
            (Some(_), None) => egg::DidMerge(false, true),
            (Some(aa), Some(bb)) => {
                if !py_eq(aa, bb) {
                    panic!("Failed to merge")
                }
                egg::DidMerge(false, false)
            }
        }
    }

    fn modify(egraph: &mut egg::EGraph<PyLang, Self>, id: egg::Id) {
        let obj = egraph[id].data.clone();
        if let Some(obj) = obj {
            let py = unsafe { Python::assume_gil_acquired() };
            let id2 = add_rec(egraph, obj.as_ref(py));
            egraph.union(id, id2);
        }
    }
}

#[pyclass(subclass)]
struct PyEGraph {
    egraph: egg::EGraph<PyLang, PyAnalysis>,
}

type Runner = egg::Runner<PyLang, PyAnalysis, ()>;

#[pymethods]
impl PyEGraph {
    #[new]
    fn new(eval: Option<PyObject>) -> Self {
        Self {
            egraph: egg::EGraph::new(PyAnalysis { eval }),
        }
    }

    fn add(&mut self, expr: &PyAny) -> PyId {
        PyId(add_rec(&mut self.egraph, expr))
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

    #[args(iter_limit = "10", time_limit = "10.0", node_limit = "100_000")]
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
            .collect::<PyResult<Vec<PyRef<PyRewrite>>>>()?;
        let egraph = std::mem::take(&mut self.egraph);
        let scheduled_runner = Runner::default();
        let runner = scheduled_runner
            .with_iter_limit(iter_limit)
            .with_node_limit(node_limit)
            .with_time_limit(Duration::from_secs_f64(time_limit))
            .with_egraph(egraph)
            .run(refs.iter().map(|r| &r.rewrite));

        self.egraph = runner.egraph;
        Ok(())
    }

    #[args(exprs = "*")]
    fn extract(&mut self, py: Python, exprs: &PyTuple) -> Vec<PyObject> {
        let ids: Vec<egg::Id> = exprs.iter().map(|expr| self.add(expr).0).collect();
        let extractor = egg::Extractor::new(&self.egraph, egg::AstSize);
        ids.iter()
            .map(|&id| {
                let (_cost, recexpr) = extractor.find_best(id);
                reconstruct(py, &recexpr)
            })
            .collect()
    }
}

fn reconstruct(py: Python, recexpr: &RecExpr<PyLang>) -> PyObject {
    let mut objs = Vec::<PyObject>::with_capacity(recexpr.as_ref().len());
    for node in recexpr.as_ref() {
        let obj = node.to_object(py, |id| objs[usize::from(id)].clone());
        objs.push(obj)
    }
    objs.pop().unwrap()
}

fn add_rec(egraph: &mut egg::EGraph<PyLang, PyAnalysis>, expr: &PyAny) -> egg::Id {
    if let Ok(PyId(id)) = expr.extract() {
        egraph.find(id)
    } else if let Ok(PyVar(var)) = expr.extract() {
        panic!("Can't add a var: {}", var)
    } else if let Ok(tuple) = expr.downcast::<PyTuple>() {
        let enode = PyLang::op(
            expr.get_type(),
            tuple.iter().map(|child| add_rec(egraph, child)),
        );
        egraph.add(enode)
    } else {
        egraph.add(PyLang::leaf(expr))
    }
}

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
