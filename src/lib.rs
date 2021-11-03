use std::hash::Hash;

use pyo3::{basic::CompareOp, prelude::*, types::PyTuple, PyObjectProtocol};

// #[pyclass(subclass)]
// #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
// struct ENode {
//     operator: egg::Symbol,
//     children: Vec<egg::Id>,
// }

#[pyclass]
#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct Id(egg::Id);

#[pyproto]
impl PyObjectProtocol for Id {
    fn __str__(&self) -> String {
        self.0.to_string()
    }
    fn __repr__(&self) -> String {
        self.0.to_string()
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

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ENode {
    Int(i64),
    Node(usize, Vec<egg::Id>),
}

impl egg::Language for ENode {
    fn matches(&self, other: &Self) -> bool {
        use ENode::*;
        match (self, other) {
            (Node(op1, args1), Node(op2, args2)) => op1 == op2 && args1.len() == args2.len(),
            (Int(a), Int(b)) => a == b,
            _ => false,
        }
    }

    fn children(&self) -> &[egg::Id] {
        match self {
            ENode::Node(_, args) => args,
            _ => &[],
        }
    }

    fn children_mut(&mut self) -> &mut [egg::Id] {
        match self {
            ENode::Node(_, args) => args,
            _ => &mut [],
        }
    }
}

#[pyclass]
#[derive(Default)]
pub struct EGraph {
    egraph: egg::EGraph<ENode, ()>,
}

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

    fn rebuild(&mut self) -> usize {
        self.egraph.rebuild()
    }
}

impl EGraph {
    fn add_rec(&mut self, expr: &PyAny) -> egg::Id {
        if let Ok(id) = expr.extract::<Id>() {
            self.egraph.find(id.0)
        } else if let Ok(tuple) = expr.downcast::<PyTuple>() {
            let ty = expr.get_type_ptr() as usize;
            let ids: Vec<egg::Id> = tuple.iter().map(|child| self.add_rec(child)).collect();
            self.egraph.add(ENode::Node(ty, ids))
        } else if let Ok(n) = expr.extract::<i64>() {
            self.egraph.add(ENode::Int(n))
        // } else if let Ok(s) = expr.extract::<&str>() {
        //     self.egraph.add(ENode::Symbol(s.into()))
        } else {
            let repr = expr.repr().expect("failed to repr");
            panic!("Cannot add {}", repr)
        }
    }
}

/// A Python module implemented in Rust. The name of this function must match
/// the `lib.name` setting in the `Cargo.toml`, else Python will not be able to
/// import the module.
#[pymodule]
fn snake_egg(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<EGraph>()?;
    m.add_class::<Id>()?;
    Ok(())
}
