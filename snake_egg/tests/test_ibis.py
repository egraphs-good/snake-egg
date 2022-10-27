import functools

import ibis
import ibis.expr.datatypes as dt
import ibis.expr.operations as ops

from snake_egg.snake_egg import EGraph, Rewrite, Var, vars

zero = ops.Literal(0, dtype=dt.int64)
one = ops.Literal(1, dtype=dt.int64)
two = ops.Literal(2, dtype=dt.float64)

x_, y_, z_, table, preds1, preds2, distinct = vars("x y z table preds1 preds2 distinct") # type: ignore

def just_x(x):
    return x


rules = [
    Rewrite(ops.Add.pattern(zero, x_), just_x, name="add-0"),
    # Rewrite(ops.Add.pattern(x_, zero), x_, name="add-0"),
    Rewrite(
        ops.Selection.pattern(
            table=table,
            selections=x_,
            predicates=ops.NodeList.pattern(),
            sort_keys=ops.NodeList.pattern(),
        ),
        table,
        name="selection-0",
    ),
    Rewrite(ops.NodeList.pattern(x_, y_, z_), ops.NodeList.pattern(x_, z_), name="node-list"),
]


def simplify(expr, iters=7):
    def reify(klass, args):
        if isinstance(klass, type):
            return klass(*args)
        else:
            return klass

    assert isinstance(expr, ops.Node), "nodes only fool"
    egraph = EGraph(reify)
    egraph.add(expr)
    egraph.run(rules, iters)
    best = egraph.extract(expr)
    return best


def test_ibis():
    assert simplify(ops.Add(zero, ops.Add(zero, two))) == two

    assert simplify(ops.NodeList(zero, one, two)) == ops.NodeList(zero, two)


# def test_union_all_to_or():
#     t = ibis.table(dict(a="int"), name="t")
#     result = simplify(expr.op())
