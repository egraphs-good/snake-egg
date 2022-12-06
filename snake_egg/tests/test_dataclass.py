#!/usr/bin/env python3

# This is a reimplementation of simple.rs from the Rust egg repository

from dataclasses import dataclass
from typing import Any
import sys
current_version = sys.version_info()
isatleast10 = current_version[1] >= 10
from snake_egg import EGraph, Rewrite, Var, vars



# Operations
@dataclass(frozen=True)
class Add:
    x: Any
    y: Any

@dataclass(frozen=True)
class Mul:
    x: Any
    y: Any


# Rewrite rules
a, b = vars("a b")  # type: ignore
if isatleast10:
    rules = [
        Rewrite(Add(a, b), Add(b, a), name="commute-add"),
        Rewrite(Mul(a, b), Mul(b, a), name="commute-mul"),
        Rewrite(Add(a, 0), a, name="add-0"),
        Rewrite(Mul(a, 0), 0, name="mul-0"),
        Rewrite(Mul(a, 1), a, name="mul-1"),
    ]
else:
    rules = []


def simplify(expr, iters=7):
    egraph = EGraph()
    egraph.add(expr)
    egraph.run(rules, iters)
    best = egraph.extract(expr)
    return best


def test_simple_1():
    assert (not isatleast10) or simplify(Mul(0, 42)) == 0


def test_simple_2():
    foo = "foo"
    assert (not isatleast10) or simplify(Add(0, Mul(1, foo))) == foo


def test_simple_3():
    assert (not isatleast10) or simplify(Mul(2, Mul(1, "foo"))) == Mul(2, "foo")


test_simple_1()
test_simple_2()
test_simple_3()
