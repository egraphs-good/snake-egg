#!/usr/bin/env python3

# This is a reimplementation of simple.rs from the Rust egg repository

from dataclasses import dataclass
from typing import Any

from snake_egg import EGraph, Rewrite, Var, vars


# Operations
class Add:
    def __init__(self, x: Any, y: Any):
        self.x = x
        self.y = y

    def __egg_head__(self):
        return self.__class__

    def __egg_args(self):
        return self.x, self.y


class Mul:
    def __init__(self, x: Any, y: Any):
        self.x = x
        self.y = y

    def __egg_head__(self):
        return self.__class__

    def __egg_args(self):
        return self.x, self.y


# Rewrite rules
a, b = vars("a b")  # type: ignore

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


def test_simple_3():
    assert simplify(Mul(2, Mul(1, "foo"))) == Mul(2, "foo")


test_simple_1()
test_simple_2()
test_simple_3()
