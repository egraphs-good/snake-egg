#!/usr/bin/env python3

# This is a reimplementation of simple.rs from the Rust egg repository

from collections import namedtuple

from egg import EGraph, Rewrite, Var, vars
from typing import NamedTuple, Any

# Operations
class Add(NamedTuple):
    x: Any
    y: Any


class Mul(NamedTuple):
    x: Any
    y: Any


# Rewrite rules
a, b = vars("a b")
rules = [
    Rewrite(Add(a, b), Add(b, a), name="commute-add"),
    Rewrite(Mul(a, b), Mul(b, a), name="commute-mul"),
    Rewrite(Add(a, 0), a, name="add-0"),
    Rewrite(Mul(a, 0), 0, name="mul-0"),
    Rewrite(Mul(a, 1), a, name="mul-1"),
]


def simplify(expr, iters=7):
    egraph = EGraph()
    egraph.add(expr)
    egraph.run(rules, iters)
    best = egraph.extract(expr)
    return best


def test_simple_1():
    assert simplify(Mul(0, 42)) == 0


def test_simple_2():
    foo = "foo"
    assert simplify(Add(0, Mul(1, foo))) == foo