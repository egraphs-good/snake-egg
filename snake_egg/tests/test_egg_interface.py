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

    def __str__(self) -> str:
        return f"Add({self.x}, {self.y})"

    @property
    def __egg_head__(self):
        return self.__class__

    @property
    def __egg_args__(self):
        return self.x, self.y


class Mul:
    def __init__(self, x: Any, y: Any):
        self.x = x
        self.y = y

    def __str__(self) -> str:
        return f"Mul({self.x}, {self.y})"

    @property
    def __egg_head__(self):
        return self.__class__

    @property
    def __egg_args__(self):
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


def is_equal(expr_a, expr_b, iters=5):
    egraph = EGraph()

    id_a = egraph.add(expr_a)
    id_b = egraph.add(expr_b)

    egraph.run(rules, iters)

    return egraph.equiv(id_a, id_b)


def test_simple_1():
    assert is_equal(Mul(0, 42), 0)


def test_simple_2():
    foo = "foo"
    assert is_equal(Add(0, Mul(1, foo)), foo)


def test_simple_3():
    foo = "foo"
    assert is_equal(Mul(2, Mul(1, foo)), Mul(2, foo))


test_simple_1()
test_simple_2()
test_simple_3()
