"""
This test is an example of using a dynamic rewrite rule.
The  test language is a basic mathematical one with:

* Symbols, represented as strings
* Numbers, represented as ints
* Addition, represented as a named tuple

We can replace addition nodes with the result, if both values are numbers.
"""
from __future__ import annotations

from collections import namedtuple
from typing import List, NamedTuple, Union, cast

from snake_egg import EGraph, Rewrite, Var, vars


class Add(NamedTuple):
    x: Expr
    y: Expr

Expr = Union[str, int, Add, Var]


def replace_add(x: Expr, y: Expr) -> Expr:
    if isinstance(x, int) and isinstance(y, int):
        return x + y
    return Add(x, y)

x, y = cast(List[Var], vars("x y"))

rules = [
    Rewrite(Add(x, y), replace_add, name="replace-add"),
]


def simplify(expr: Expr):
    egraph = EGraph()
    egraph.add(expr)
    egraph.run(rules)
    best = egraph.extract(expr)
    return best


def test_simplify_add():
    assert simplify(Add(1, 2)) == 3
    assert simplify(Add(1, Add("x", "y"))) == Add(1, Add("x", "y"))

