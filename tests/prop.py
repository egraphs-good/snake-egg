#!/usr/bin/env python3

# This is a reimplementation of simple.rs from the Rust egg repository

from snake_egg import EGraph, Rewrite, Var, vars

import unittest

from collections import namedtuple


# Operations
AND = namedtuple("And", "x y")
NOT = namedtuple("Not", "x")
OR  = namedtuple("Or", "x y")
IM  = namedtuple("Implies", "x y")

# Allow constant folding via an eval function
def eval_prod(car, cdr):

    # This could be a literal
    if type(car) == bool:
        return car

    # Or a variable
    if len(cdr) == 0:
        return None

    # Else it is an operation with arguments
    op = car
    args = cdr

    # Symbolic values cannot be evaluated
    if any(type(a) != bool for a in args):
        return None

    a = args[0]
    if op == NOT:
        return not a

    b = args[1]
    if op == AND:
        return a and b

    if op == OR:
        return a or b

    if op == IM:
        return a or not b

    return None


# Rewrite rules
a, b, c = vars("a b c")
list_rules = [
  ["def_imply",        IM(a, b),                      OR(NOT(a), b)],
  ["double_neg",       NOT(NOT(a)),                   a],
  ["def_imply_flip",   OR(NOT(a), b),                 IM(a, b)],
  ["double_neg_flip",  a,                             NOT(NOT(a))],
  ["assoc_or",         OR(a, OR(b, c)),               OR(OR(a, b), c)],
  ["dist_and_or",      AND(a, OR(b, c)),              OR(AND(a, b), AND(a, c))],
  ["dist_or_and",      OR(a, AND(b, c)),              AND(OR(a, b), OR(a, c))],
  ["comm_or",          OR(a, b),                      OR(b, a)],
  ["comm_and",         AND(a, b),                     AND(b, a)],
  ["lem",              OR(a, NOT(a)),                 True],
  ["or_true",          OR(a, True),                   True],
  ["and_true",         AND(a, True),                  a],
  ["contrapositive",   IM(a, b),                      IM(NOT(b), NOT(a))],
  ["lem_imply",        AND(IM(a, b), IM(NOT(a), c)),  OR(b, c)],
]

# Turn the lists into rewrites
rules = list()
for l in list_rules:
    name = l[0]
    frm = l[1]
    to = l[2]
    rules.append(Rewrite(frm, to, name))


def prove_something(start_expr, goal_exprs, tester):
    egraph = EGraph(eval_prod)
    id_start = egraph.add(start_expr)
    egraph.run(rules, 10)
    for i,goal in enumerate(goal_exprs):
        id_goal = egraph.add(goal)
        tester.assertTrue(egraph.equiv(id_start, id_goal),
                          "Couldn't prove goal {}: {}".format(i, goal))


x = "x"
y = "y"
z = "z"

class TestPropEgraph(unittest.TestCase):

    def test_prove_contrapositive(self):
        prove_something(IM(x,y),
                        [IM(x,y),
                         OR(NOT(x), y),
                         OR(NOT(x), NOT(NOT(y))),
                         OR(NOT(NOT(y)), NOT(x)),
                         IM(NOT(y), NOT(x))],
                        self)

    def test_prove_chain(self):
        prove_something(AND(IM(x, y), IM(y, z)),
                        [AND(IM(x, y), IM(y, z)),
                         AND(IM(NOT(y), NOT(x)), IM(y, z)),
                         AND(IM(y, z), IM(NOT(y), NOT(x))),
                         OR(z, NOT(x)),
                         OR(NOT(x), z),
                         IM(x, z)],
                        self)

    def test_prove_fold(self):
        prove_something(OR(AND(False, True), AND(True, False)),
                        [False],
                        self)


if __name__ == '__main__':
    unittest.main(verbosity=2)
