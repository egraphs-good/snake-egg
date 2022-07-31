#!/usr/bin/env python3

# This is a reimplementation of simple.rs from the Rust egg repository

import unittest
from collections import namedtuple
from typing import Any

from snake_egg import EGraph, Rewrite, vars

# Operations
Add = namedtuple("Add", "x y")
Mul = namedtuple("Mul", "x y")


# Rewrite rules
a, b = vars("a b")  # type: ignore
list_rules: list[list[Any]] = [
    ["commute-add", Add(a, b), Add(b, a)],
    ["commute-mul", Mul(a, b), Mul(b, a)],
    ["add-0", Add(a, 0), a],
    ["mul-0", Mul(a, 0), 0],
    ["mul-1", Mul(a, 1), a],
]

# Turn the lists into rewrites
rules = list[Rewrite]()
for l in list_rules:
    name = l[0]
    frm = l[1]
    to = l[2]
    rules.append(Rewrite(frm, to, name))


def simplify(expr, iters=7):
    egraph = EGraph()
    egraph.add(expr)
    egraph.run(rules, iters)
    best = egraph.extract(expr)
    return best


class TestSimpleEgraph(unittest.TestCase):
    def test_simple_1(self):
        self.assertEqual(simplify(Mul(0, 42)), 0)

    def test_simple_2(self):
        foo = "foo"
        self.assertEqual(simplify(Add(0, Mul(1, foo))), foo)


if __name__ == "__main__":
    unittest.main(verbosity=2)
