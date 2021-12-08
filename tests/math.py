#!/usr/bin/env python3

import unittest

from snake_egg import EGraph, Rewrite, Var, vars

from collections import namedtuple



diff = namedtuple("Diff", "x y")
inte = namedtuple("Integral", "x y")

add  = namedtuple("Add", "x y")
sub  = namedtuple("Sub", "x y")
mul  = namedtuple("Mul", "x y")
div  = namedtuple("Div", "x y")
pow  = namedtuple("Pow", "x y")
ln   = namedtuple("Ln", "x")
sqrt = namedtuple("Sqrt", "x")

sin  = namedtuple("Sin", "x")
cos  = namedtuple("Cos", "x")


a, b, c, x, f, g, y = vars("a b c x f g y")

list_rules = [
    # name, from, to, [condition,]*
    ["comm-add",      add(a, b),                 add(b, a)],
    ["comm-mul",      mul(a, b),                 mul(b, a)],
    ["assoc-add",     add(a, add(b, c)),         add(add(a, b), c)],
    ["assoc-mul",     mul(a, mul(b, c)),         mul(mul(a, b), c)],
    ["sub-canon",     sub(a, b),                 add(a, mul(-1, b))],
    #["div-canon",     div(a, b),                 mul(a, pow(b, -1)),             is_not_zero("b")],
    ["zero-add",      add(a, 0),                 a],
    ["zero-mul",      mul(a, 0),                 0],
    ["one-mul",       mul(a, 1),                 a],
    ["add-zero",      a,                         add(a, 0)],
    ["mul-one",       a,                         mul(a, 1)],
    ["cancel-sub",    sub(a, a),                 0],
    #["cancel-div",    div(a, a),                 1, is_not_zero("a")],
    ["distribute",    mul(a, add(b, c)),         add(mul(a, b), mul(a, c))],
    ["factor",        add(mul(a, b), mul(a, c)), mul(a, add(b, c))],
    ["pow-mul",       mul(pow(a, b), pow(a, c)), pow(a, add(b, c))],
    #["pow0",          pow(x, 0),                 1,                  is_not_zero(x)],
    ["pow1",          pow(x, 1),                 x],
    ["pow2",          pow(x, 2),                 mul(x, x)],
    #["pow-recip",     pow(x, -1),                div(1 x), is_not_zero(x)],
    #["recip-mul-div", mul(x, div(1 x)),          1, is_not_zero(x)],
    #["d-variable",    diff(x, x),                1, is_sym(x)],
    #["d-constant",    diff(x, c),                0, is_sym(x),                  is_const_or_distinct_var("c", x)],
    ["d-add",         diff(x, add(a, b)),        add(diff(x, a), diff(x, b))],
    ["d-mul",         diff(x, mul(a, b)),        add(mul(a, diff(x, b)), mul(b, diff(x, a)))],
    ["d-sin",         diff(x, sin(x)),           cos(x)],
    ["d-cos",         diff(x, cos(x)),           mul(-1, sin(x))],
    #["d-ln",          diff(x, ln(x)),           div(1 x), is_not_zero(x)],
    ["i-one",         inte(1, x),                x],
    #["i-power-const", inte(pow(x, c), x),        div(pow(x, add(c 1)) add(c 1)), is_const("c")],
    ["i-cos",         inte(cos(x), x),           sin(x)],
    ["i-sin",         inte(sin(x), x),           mul(-1, cos(x))],
    ["i-sum",         inte(add(f, g), x),        add(inte(f, x), inte(g, x))],
    ["i-d,",          inte(sub(f, g), x),        sub(inte(f, x), inte(g, x))],
    ["i-parts",       inte(mul(a, b), x),        sub(mul(a, inte(b, x)), inte(mul(diff(x, a), inte(b, x)), x))],

    # ["d-power",
    #  diff(x, pow(f, g)),
    #  mul(pow(f, g)
    #      add(mul(diff(x, f)
    #              div(g, f))
    #          mul(diff(x, g)
    #              ln(f))))
    #  , is_not_zero("f")
    #  , is_not_zero("g")
    #  ],

]

rules = list()
for l in list_rules:
    name = l[0]
    frm = l[1]
    to = l[2]
    rules.append(Rewrite(frm, to, name))

a = "a"
x = "x"
y = "y"


def is_equal(expr_a, expr_b, iters=7):
    egraph = EGraph()

    id_a = egraph.add(expr_a)
    id_b = egraph.add(expr_b)

    egraph.run(rules, iters)

    return egraph.equiv(id_a, id_b)


class TestMathEgraph(unittest.TestCase):

    def test_math_associate_adds(self):
        expr_a = add(1, add(2, add(3, add(4, add(5, add(6, 7))))))
        expr_b = add(7, add(6, add(5, add(4, add(3, add(2, 1))))))
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_math_simplify_add(self):
        expr_a = add(x, add(x, add(x, x)))
        expr_b = mul(4, x)
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_math_powers(self):
        expr_a = mul(pow(2, x), pow(2, y))
        expr_b = pow(2, add(x, y))
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_math_simplify_const(self):
        expr_a = add(1, sub(a, mul(sub(2, 1), a)))
        expr_b = 1
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_math_simplify_root(self):
        expr_a = div(1, sub(div(add(1, sqrt(5)), 2), div(sub(1, sqrt(5)), 2)))
        expr_b = div(1, sqrt(5))
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_math_simplify_factor(self):
        expr_a = mul(add(x, 3), add(x, 1))
        expr_b = add(add(mul(x, x), mul(4, x)), 3)
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_math_diff_same(self):
        expr_a = diff(x, x)
        expr_b = 1
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_math_diff_different(self):
        expr_a = diff(x, y)
        expr_b = 0
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_math_diff_simple1(self):
        expr_a = diff(x, add(1, mul(2, x)))
        expr_b = 2
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_math_diff_simple2(self):
        expr_a = diff(x, add(1, mul(y, x)))
        expr_b = y
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_math_diff_ln(self):
        expr_a = diff(x, ln(x))
        expr_b = div(1, x)
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_diff_power_simple(self):
        expr_a = diff(x, pow(x, 3))
        expr_b = mul(3, pow(x, 2))
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_diff_power_harder(self):
        expr_a = diff(x, sub(pow(x, 3), mul(7, pow(x, 2))))
        expr_b = mul(x, sub(mul(3, x), 14))
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_integ_one(self):
        expr_a = inte(1, x)
        expr_b = x
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_integ_sin(self):
        expr_a = inte(cos(x), x)
        expr_b = sin(x)
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_integ_x(self):
        expr_a = inte(pow(x, 1), x)
        expr_b = div(pow(x, 2), 2)
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_integ_part1(self):
        expr_a = inte(mul(x, cos(x)), x)
        expr_b = add(mul(x, sin(x)), cos(x))
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_integ_part2(self):
        expr_a = inte(mul(cos(x), x), x)
        expr_b = add(mul(x, sin(x)), cos(x))
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_integ_part3(self):
        expr_a = inte(ln(x), x)
        expr_b = sub(mul(x, ln(x)), x)
        self.assertTrue(is_equal(expr_a, expr_b))




if __name__ == '__main__':
    unittest.main(verbosity=2)
