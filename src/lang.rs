use egg::{Analysis, Applier, DidMerge, EGraph, PatternAst, Subst, Symbol};
use egg::{Id, Language};
use once_cell::sync::Lazy;
use pyo3::AsPyPointer;
use pyo3::{
    basic::CompareOp,
    prelude::*,
    types::{PyTuple, PyType, PyDict},
};
use std::cmp::Ordering;
use std::sync::Mutex;
use std::{fmt::Display, hash::Hash};

use crate::util::{build_node, py_eq};
use crate::core::PyPattern;

struct PythonHashable {
    obj: PyObject,
    hash: isize,
}

impl PythonHashable {
    pub fn new(obj: &PyAny) -> Self {
        Self {
            obj: obj.into(),
            hash: obj.hash().expect("Failed to hash"),
        }
    }
}

impl Hash for PythonHashable {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.hash.hash(state);
    }
}

impl PartialEq for PythonHashable {
    fn eq(&self, other: &Self) -> bool {
        let py = unsafe { Python::assume_gil_acquired() };

        self.obj
            .as_ref(py)
            .rich_compare(&other.obj, CompareOp::Eq)
            .expect("Failed to compare")
            .is_true()
            .expect("Failed to extract bool")
    }
}

impl Eq for PythonHashable {}

#[derive(Debug, Clone)]
pub struct PythonNode {
    pub class: PyObject,
    pub children: Vec<Id>,
}

impl PythonNode {
    pub fn op(ty: &PyType, children: impl IntoIterator<Item = Id>) -> Self {
        Self {
            class: ty.into(),
            children: children.into_iter().collect(),
        }
    }

    pub fn leaf(obj: &PyAny) -> Self {
        static LEAVES: Lazy<Mutex<hashbrown::HashSet<PythonHashable>>> =
            Lazy::new(Default::default);
        let mut leaves = LEAVES.lock().unwrap();

        let object = PythonHashable::new(obj);
        let hashable = leaves.get_or_insert(object);

        Self {
            class: hashable.obj.clone(),
            children: vec![],
        }
    }

    pub fn to_object<T: IntoPy<PyObject>>(&self, py: Python, f: impl FnMut(Id) -> T) -> PyObject {
        if self.is_leaf() {
            self.class.clone()
        } else {
            let children = self.children.iter().copied().map(f);
            let args = PyTuple::new(py, children.map(|o| o.into_py(py)));
            self.class.call1(py, args).expect("Failed to construct")
        }
    }
}

impl Language for PythonNode {
    fn matches(&self, other: &Self) -> bool {
        self.class.as_ptr() == other.class.as_ptr() && self.children.len() == other.children.len()
    }

    fn children(&self) -> &[Id] {
        &self.children
    }

    fn children_mut(&mut self) -> &mut [Id] {
        &mut self.children
    }
}

impl PartialEq for PythonNode {
    fn eq(&self, other: &Self) -> bool {
        self.class.as_ptr() == other.class.as_ptr() && self.children == other.children
    }
}

impl Hash for PythonNode {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.class.as_ptr().hash(state);
        self.children.hash(state);
    }
}

impl Ord for PythonNode {
    fn cmp(&self, other: &Self) -> Ordering {
        self.partial_cmp(other).expect("comparison failed")
    }
}

impl PartialOrd for PythonNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        match self.class.as_ptr().partial_cmp(&other.class.as_ptr()) {
            Some(Ordering::Equal) => {}
            ord => return ord,
        }
        self.children.partial_cmp(&other.children)
    }
}

impl Eq for PythonNode {}

impl Display for PythonNode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Python::with_gil(|py| match self.class.as_ref(py).str() {
            Ok(s) => s.fmt(f),
            Err(_) => "<<NODE>>".fmt(f),
        })
    }
}

#[derive(Default)]
pub struct PythonAnalysis {
    pub eval: Option<PyObject>,
}

impl Analysis<PythonNode> for PythonAnalysis {
    type Data = Option<PyObject>;

    fn make(egraph: &EGraph<PythonNode, Self>, enode: &PythonNode) -> Self::Data {
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
            .call1(py, (enode.class.clone(), children))
            .expect("Failed to call eval");
        if res.is_none(py) {
            None
        } else {
            Some(res)
        }
    }

    fn merge(&mut self, a: &mut Self::Data, b: Self::Data) -> DidMerge {
        let py = unsafe { Python::assume_gil_acquired() };
        let aa = a.as_ref().map(|obj| obj.as_ref(py)).filter(|r| r.is_none());
        let bb = b.as_ref().map(|obj| obj.as_ref(py)).filter(|r| r.is_none());
        match (aa, bb) {
            (None, None) => DidMerge(false, false),
            (None, Some(bb)) => {
                *a = Some(bb.to_object(py));
                DidMerge(true, false)
            }
            (Some(_), None) => egg::DidMerge(false, true),
            (Some(aa), Some(bb)) => {
                if !py_eq(aa, bb) {
                    panic!("Failed to merge")
                }
                DidMerge(false, false)
            }
        }
    }

    fn modify(egraph: &mut EGraph<PythonNode, Self>, id: Id) {
        let obj = egraph[id].data.clone();
        if let Some(obj) = obj {
            let py = unsafe { Python::assume_gil_acquired() };
            let id2 = build_node(egraph, obj.as_ref(py));
            egraph.union(id, id2);
        }
    }
}

pub struct PythonApplier {
    pub eval: PyObject,
}

impl Applier<PythonNode, PythonAnalysis> for PythonApplier {

    fn apply_one(
        &self,
        egraph: &mut EGraph<PythonNode, PythonAnalysis>,
        eclass: Id,
        subst: &Subst,
        searcher_ast: Option<&PatternAst<PythonNode>>,
        rule_name: Symbol,
    ) -> Vec<Id> {
        let py = unsafe { Python::assume_gil_acquired() };
        let kwargs = PyDict::new(py);

        for (var, id) in subst.vec.iter() {
            let obj = if let Some(data) = egraph[*id].data.clone() {
                data
            } else {
                py.None()
            };
            let key = &var.to_string()[1..];
            kwargs.set_item(key, obj).unwrap();
        }

        let result = self.eval.as_ref(py).call((), Some(kwargs)).unwrap();
        let pattern = result.extract::<PyPattern>().unwrap();
        pattern.pattern.apply_one(egraph, eclass, subst, searcher_ast, rule_name)

    }
}
