#!/usr/bin/env python3

# This is a reimplementation of simple.rs from the Rust egg repository

from snake_egg import EGraph, Rewrite, Var, vars

import unittest
from typing import List, Any
from collections import namedtuple


# Operations
add  = namedtuple("Add", "x y") # type: ignore
mul  = namedtuple("Mul", "x y") # type: ignore


# Rewrite rules
a, b = vars("a b") # type: ignore
list_rules: List[List[Any]] = [
  ["commute-add",  add(a, b),  add(b, a)],
  ["commute-mul",  mul(a, b),  mul(b, a)],
  ["add-0",        add(a, 0),  a],
  ["mul-0",        mul(a, 0),  0],
  ["mul-1",        mul(a, 1),  a],
]

# Turn the lists into rewrites
rules = list()
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
        self.assertEqual(simplify(mul(0, 42)), 0)

    def test_simple_2(self):
        foo = "foo"
        self.assertEqual(simplify(add(0, mul(1, foo))), foo)


if __name__ == '__main__':
    unittest.main(verbosity=2)