#!/usr/bin/env python3

# This is a reimplementation of math.rs from the Rust egg repository
# Things not present in this version:
# * custom cost function
# * analysis / guards for rules
#   + this disables some rules and tests
# * the last three tests

from snake_egg import EGraph, Rewrite, Var, vars

import unittest

from collections import namedtuple


# Operations
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

# Allow constant folding via an eval function
def eval_math(car, cdr):
    # This could be a literal encoded in a string
    try:
        return float(car)
    except:
        pass

    # Else it is an operation with arguments
    op = car
    args = cdr
    try:
        a = float(args[0])
        b = float(args[1])
        if op == add:
            return a + b
        if op == sub:
            return a - b
        if op == mul:
            return a * b
        if op == div and b != 0.0:
            return a / b
    except:
        pass
    return None

# Rewrite rules, not all are currently used since gaurds aren't in snake-egg yet
a, b, c, x, f, g, y = vars("a b c x f g y")
list_rules = [
  # name,        from,               to
  ["comm-add",   add(a, b),          add(b, a)],
  ["comm-mul",   mul(a, b),          mul(b, a)],
  ["assoc-add",  add(a, add(b, c)),  add(add(a, b), c)],
  ["assoc-mul",  mul(a, mul(b, c)),  mul(mul(a, b), c)],

  ["sub-canon",  sub(a, b),  add(a, mul(-1, b))],
  # rw!("div-canon"; "(/ ?a ?b)" => "(* ?a (pow ?b -1))" if is_not_zero("?b")),
  # // rw!("canon-sub"; "(+ ?a (* -1 ?b))"  => "(- ?a ?b)"),
  # // rw!("canon-div"; "(* ?a (pow ?b -1))" => "(/ ?a ?b)" if is_not_zero("?b")),

  ["zero-add",  add(a, 0),  a],
  ["zero-mul",  mul(a, 0),  0],
  ["one-mul",   mul(a, 1),  a],

  ["add-zero",  a,  add(a, 0)],
  ["mul-one",   a,  mul(a, 1)],

  ["cancel-sub",  sub(a, a),  0],
  # rw!("cancel-div"; "(/ ?a ?a)" => "1" if is_not_zero("?a")),

  ["distribute",  mul(a, add(b, c)),          add(mul(a, b), mul(a, c))],
  ["factor",      add(mul(a, b), mul(a, c)),  mul(a, add(b, c))],

  ["pow-mul",  mul(pow(a, b), pow(a, c)),  pow(a, add(b, c))],
  # rw!("pow0"; "(pow ?x 0)" => "1"
  #            if is_not_zero("?x")),
  ["pow1",     pow(x, 1),                  x],
  ["pow2",     pow(x, 2),                  mul(x, x)],
  # rw!("pow-recip"; "(pow ?x -1)" => "(/ 1 ?x)"
  #            if is_not_zero("?x")),
  # rw!("recip-mul-div"; "(* ?x (/ 1 ?x))" => "1" if is_not_zero("?x")),

  # rw!("d-variable"; "(d ?x ?x)" => "1" if is_sym("?x")),
  # rw!("d-constant"; "(d ?x ?c)" => "0" if is_sym("?x") if is_const_or_distinct_var("?c", "?x")),

  ["d-add",  diff(x, add(a, b)),  add(diff(x, a), diff(x, b))],
  ["d-mul",  diff(x, mul(a, b)),  add(mul(a, diff(x, b)), mul(b, diff(x, a)))],

  ["d-sin",  diff(x, sin(x)),  cos(x)],
  ["d-cos",  diff(x, cos(x)),  mul(-1, sin(x))],

  # rw!("d-ln"; "(d ?x (ln ?x))" => "(/ 1 ?x)" if is_not_zero("?x")),

  # rw!("d-power";
  #     "(d ?x (pow ?f ?g))" =>
  #     "(* (pow ?f ?g)
  #         (+ (* (d ?x ?f)
  #               (/ ?g ?f))
  #            (* (d ?x ?g)
  #               (ln ?f))))"
  #     if is_not_zero("?f")
  #     if is_not_zero("?g")
  # ),

  ["i-one",    inte(1, x),          x],
  # rw!("i-power-const"; "(i (pow ?x ?c) ?x)" =>
  #            "(/ (pow ?x (+ ?c 1)) (+ ?c 1))" if is_const("?c")),
  ["i-cos",    inte(cos(x), x),     sin(x)],
  ["i-sin",    inte(sin(x), x),     mul(-1, cos(x))],
  ["i-sum",    inte(add(f, g), x),  add(inte(f, x), inte(g, x))],
  ["i-dif",    inte(sub(f, g), x),  sub(inte(f, x), inte(g, x))],
  ["i-parts",  inte(mul(a, b), x),
        sub(mul(a, inte(b, x)), inte(mul(diff(x, a), inte(b, x)), x))],
]

# Turn the lists into rewrites
rules = list()
for l in list_rules:
    name = l[0]
    frm = l[1]
    to = l[2]
    rules.append(Rewrite(frm, to, name))


a = "a"
x = "x"
y = "y"
five = "five"

def is_equal(expr_a, expr_b, iters=7):
    egraph = EGraph(eval_math)

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

    # def test_math_simplify_root(self):
    #     expr_a = div(1, sub(div(add(1, sqrt(five)), 2),
    #                         div(sub(1, sqrt(five)), 2)))
    #     expr_b = div(1, sqrt(five))
    #     self.assertTrue(is_equal(expr_a, expr_b))

    def test_math_simplify_factor(self):
        expr_a = mul(add(x, 3), add(x, 1))
        expr_b = add(add(mul(x, x), mul(4, x)), 3)
        self.assertTrue(is_equal(expr_a, expr_b))

    # def test_math_diff_same(self):
    #     expr_a = diff(x, x)
    #     expr_b = 1
    #     self.assertTrue(is_equal(expr_a, expr_b))

    # def test_math_diff_different(self):
    #     expr_a = diff(x, y)
    #     expr_b = 0
    #     self.assertTrue(is_equal(expr_a, expr_b))

    # def test_math_diff_simple1(self):
    #     expr_a = diff(x, add(1, mul(2, x)))
    #     expr_b = 2
    #     self.assertTrue(is_equal(expr_a, expr_b))

    # def test_math_diff_simple2(self):
    #     expr_a = diff(x, add(1, mul(y, x)))
    #     expr_b = y
    #     self.assertTrue(is_equal(expr_a, expr_b))

    # def test_math_diff_ln(self):
    #     expr_a = diff(x, ln(x))
    #     expr_b = div(1, x)
    #     self.assertTrue(is_equal(expr_a, expr_b))

    # def test_diff_power_simple(self):
    #     expr_a = diff(x, pow(x, 3))
    #     expr_b = mul(3, pow(x, 2))
    #     self.assertTrue(is_equal(expr_a, expr_b))

    # def test_diff_power_harder(self):
    #     expr_a = diff(x, sub(pow(x, 3), mul(7, pow(x, 2))))
    #     expr_b = mul(x, sub(mul(3, x), 14))
    #     self.assertTrue(is_equal(expr_a, expr_b))

    def test_integ_one(self):
        expr_a = inte(1, x)
        expr_b = x
        self.assertTrue(is_equal(expr_a, expr_b))

    def test_integ_sin(self):
        expr_a = inte(cos(x), x)
        expr_b = sin(x)
        self.assertTrue(is_equal(expr_a, expr_b))

    # def test_integ_x(self):
    #     expr_a = inte(pow(x, 1), x)
    #     expr_b = div(pow(x, 2), 2)
    #     self.assertTrue(is_equal(expr_a, expr_b))

    # def test_integ_part1(self):
    #     expr_a = inte(mul(x, cos(x)), x)
    #     expr_b = add(mul(x, sin(x)), cos(x))
    #     self.assertTrue(is_equal(expr_a, expr_b))

    # def test_integ_part2(self):
    #     expr_a = inte(mul(cos(x), x), x)
    #     expr_b = add(mul(x, sin(x)), cos(x))
    #     self.assertTrue(is_equal(expr_a, expr_b))

    # def test_integ_part3(self):
    #     expr_a = inte(ln(x), x)
    #     expr_b = sub(mul(x, ln(x)), x)
    #     self.assertTrue(is_equal(expr_a, expr_b))




if __name__ == '__main__':
    unittest.main(verbosity=2)
