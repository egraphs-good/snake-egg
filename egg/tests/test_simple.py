#!/usr/bin/env python3

# This is a reimplementation of simple.rs from the Rust egg repository

from collections import namedtuple

from egg import EGraph, Rewrite, Var, vars

# Operations
add = namedtuple("Add", "x y")
mul = namedtuple("Mul", "x y")


# Rewrite rules
a, b = vars("a b")
list_rules = [
    ["commute-add", add(a, b), add(b, a)],
    ["commute-mul", mul(a, b), mul(b, a)],
    ["add-0", add(a, 0), a],
    ["mul-0", mul(a, 0), 0],
    ["mul-1", mul(a, 1), a],
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


def test_simple_1():
    assert simplify(mul(0, 42)) == 0


def test_simple_2():
    foo = "foo"
    assert simplify(add(0, mul(1, foo))) == foo
