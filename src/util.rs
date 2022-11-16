use egg::{EGraph, ENodeOrVar, Id, PatternAst};
use pyo3::types::{PyTuple, PyType};
use pyo3::{basic::CompareOp, prelude::*};

use crate::{PyId, PyVar, PythonAnalysis, PythonNode};

pub fn py_eq(a: &PyAny, b: impl ToPyObject) -> bool {
    a.rich_compare(b, CompareOp::Eq)
        .expect("Failed to compare")
        .is_true()
        .expect("Failed to extract bool")
}

// TODO(kszucs): proper error handling
pub fn build_node(egraph: &mut EGraph<PythonNode, PythonAnalysis>, expr: &PyAny) -> Id {
    if let Ok(PyId(id)) = expr.extract() {
        egraph.find(id)
    } else if let Ok(PyVar(var)) = expr.extract() {
        panic!("Can't add a var: {}", var)
    } else if let Ok(args) = expr.getattr("__match_args__") {
        let args = args.downcast::<PyTuple>().unwrap();
        let class = if let Ok(class) = expr.getattr("__match_type__") {
            class.downcast::<PyType>().unwrap()
        } else {
            expr.get_type()
        };
        //let children = args.iter().map(|arg| expr.getattr(arg).unwrap());
        let enode = PythonNode::op(class, args.iter().map(|child| build_node(egraph, child)));
        egraph.add(enode)
    } else if let Ok(tuple) = expr.downcast::<PyTuple>() {
        let enode = PythonNode::op(
            expr.get_type(),
            tuple.iter().map(|child| build_node(egraph, child)),
        );
        egraph.add(enode)
    } else {
        egraph.add(PythonNode::leaf(expr))
    }
}

// TODO(kszucs): proper error handling
pub fn build_pattern(ast: &mut PatternAst<PythonNode>, tree: &PyAny) -> Id {
    if let Ok(id) = tree.extract::<PyId>() {
        panic!("Ids are unsupported in patterns: {}", id.0)
    } else if let Ok(var) = tree.extract::<PyVar>() {
        ast.add(ENodeOrVar::Var(var.0))
    // check for Sequence first?
    } else if let Ok(args) = tree.getattr("__match_args__") {
        let args = args.downcast::<PyTuple>().unwrap();
        let class = if let Ok(class) = tree.getattr("__match_type__") {
            class.downcast::<PyType>().unwrap()
        } else {
            tree.get_type()
        };
        //let children = args.iter().map(|arg| tree.getattr(arg).unwrap());
        let enode = PythonNode::op(class, args.iter().map(|child| build_pattern(ast, child)));
        ast.add(ENodeOrVar::ENode(enode))
    } else if let Ok(tuple) = tree.downcast::<PyTuple>() {
        let enode = PythonNode::op(
            tree.get_type(),
            tuple.iter().map(|child| build_pattern(ast, child)),
        );
        ast.add(ENodeOrVar::ENode(enode))
    } else {
        ast.add(ENodeOrVar::ENode(PythonNode::leaf(tree)))
    }
}
