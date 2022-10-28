#!/usr/bin/env python3

# This is a reimplementation of simple.rs from the Rust egg repository

from collections import namedtuple
from typing import Any, List

from snake_egg import EGraph, Rewrite, vars

# Operations
And = namedtuple("And", "x y")
Not = namedtuple("Not", "x")
Or  = namedtuple("Or", "x y")
Implies = namedtuple("Implies", "x y")

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
    if op == Not:
        return not a

    b = args[1]
    if op == And:
        return a and b

    if op == Or:
        return a or b

    if op == Implies:
        return a or not b

    return None


# Rewrite rules
a, b, c = vars("a b c") # type: ignore
list_rules: List[List[Any]] = [
  ["def_imply",        Implies(a, b),                 Or(Not(a), b)],
  ["double_neg",       Not(Not(a)),                   a],
  ["def_imply_flip",   Or(Not(a), b),                 Implies(a, b)],
  ["double_neg_flip",  a,                             Not(Not(a))],
  ["assoc_or",         Or(a, Or(b, c)),               Or(Or(a, b), c)],
  ["dist_and_or",      And(a, Or(b, c)),              Or(And(a, b), And(a, c))],
  ["dist_or_and",      Or(a, And(b, c)),              And(Or(a, b), Or(a, c))],
  ["comm_or",          Or(a, b),                      Or(b, a)],
  ["comm_and",         And(a, b),                     And(b, a)],
  ["lem",              Or(a, Not(a)),                 True],
  ["or_true",          Or(a, True),                   True],
  ["and_true",         And(a, True),                  a],
  ["contrapositive",   Implies(a, b),                 Implies(Not(b), Not(a))],
  ["lem_imply",        And(Implies(a, b), Implies(Not(a), c)),  Or(b, c)],
]
# fmt: on

# Turn the lists into rewrites
rules = list()
for l in list_rules:
    name = l[0]
    frm = l[1]
    to = l[2]
    rules.append(Rewrite(frm, to, name))


def prove_something(start_expr, goal_exprs):
    egraph = EGraph(eval_prod)
    id_start = egraph.add(start_expr)
    egraph.run(rules, 10)
    for i, goal in enumerate(goal_exprs):
        id_goal = egraph.add(goal)
        assert egraph.equiv(id_start, id_goal), "Couldn't prove goal {}: {}".format(
            i, goal
        )


x = "x"
y = "y"
z = "z"


def test_prove_contrapositive():
    prove_something(Implies(x,y),
                    [Implies(x,y),
                        Or(Not(x), y),
                        Or(Not(x), Not(Not(y))),
                        Or(Not(Not(y)), Not(x)),
                        Implies(Not(y), Not(x))])

def test_prove_chain():
    prove_something(And(Implies(x, y), Implies(y, z)),
                    [And(Implies(x, y), Implies(y, z)),
                        And(Implies(Not(y), Not(x)), Implies(y, z)),
                        And(Implies(y, z), Implies(Not(y), Not(x))),
                        Or(z, Not(x)),
                        Or(Not(x), z),
                        Implies(x, z)])

def test_prove_fold():
    prove_something(Or(And(False, True), And(True, False)),
                    [False])

