use egg::{AstSize, EGraph, Extractor, Id, Pattern, PatternAst, RecExpr, Rewrite, Runner, Var};
use pyo3::types::{PyList, PyString, PyTuple};
use pyo3::{basic::CompareOp, prelude::*};

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::time::Duration;

use crate::lang::{PythonAnalysis, PythonApplier, PythonNode};
use crate::util::{build_node, build_pattern};

#[pyclass]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyId(pub Id);

#[pymethods]
impl PyId {
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

#[pyclass]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PyVar(pub Var);

#[pymethods]
impl PyVar {
    #[new]
    fn new(str: &PyString) -> Self {
        Self::from_str(str.to_string_lossy().as_ref())
    }

    fn __hash__(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        self.0.hash(&mut hasher);
        hasher.finish().into()
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

impl PyVar {
    pub fn from_str(str: &str) -> Self {
        let v = format!("?{}", str);
        PyVar(v.parse().unwrap())
    }
}

#[pyclass]
pub struct PyPattern {
    pub pattern: Pattern<PythonNode>,
}

#[pyclass]
pub struct PyRewrite {
    pub rewrite: Rewrite<PythonNode, PythonAnalysis>,
}

#[pymethods]
impl PyRewrite {
    #[new]
    #[args(name = "\"\"")]
    fn new(searcher: PyPattern, applier: &PyAny, name: &str) -> Self {
        let rewrite = if applier.is_callable() {
            let applier = PythonApplier { eval: applier.into() };
            Rewrite::new(name, searcher.pattern, applier).unwrap()
        } else if let Ok(pat) = applier.extract::<PyPattern>() {
            Rewrite::new(name, searcher.pattern, pat.pattern).unwrap()
        } else {
            panic!("Applier must be a pattern or callable");
        };
        PyRewrite { rewrite }
    }

    #[getter]
    fn name(&self) -> &str {
        self.rewrite.name.as_str()
    }
}

impl<'source> FromPyObject<'source> for PyPattern {
    fn extract(obj: &'source PyAny) -> PyResult<Self> {
        let mut ast = PatternAst::default();
        build_pattern(&mut ast, obj);
        let pattern = Pattern::from(ast);
        Ok(Self { pattern })
    }
}

#[pyclass(subclass)]
pub struct PyEGraph {
    pub egraph: EGraph<PythonNode, PythonAnalysis>,
}

#[pymethods]
impl PyEGraph {
    #[new]
    fn new(eval: Option<PyObject>) -> Self {
        Self {
            egraph: EGraph::new(PythonAnalysis { eval }),
        }
    }

    fn add(&mut self, expr: &PyAny) -> PyId {
        PyId(build_node(&mut self.egraph, expr))
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
        let scheduled_runner = Runner::<PythonNode, PythonAnalysis>::default();
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
        let ids: Vec<Id> = exprs.iter().map(|expr| self.add(expr).0).collect();
        let extractor = Extractor::new(&self.egraph, AstSize);
        ids.iter()
            .map(|&id| {
                let (_cost, recexpr) = extractor.find_best(id);
                reconstruct(py, &recexpr)
            })
            .collect()
    }
}

fn reconstruct(py: Python, recexpr: &RecExpr<PythonNode>) -> PyObject {
    let mut objs = Vec::<PyObject>::with_capacity(recexpr.as_ref().len());
    for node in recexpr.as_ref() {
        let obj = node.to_object(py, |id| objs[usize::from(id)].clone());
        objs.push(obj)
    }
    objs.pop().unwrap()
}
