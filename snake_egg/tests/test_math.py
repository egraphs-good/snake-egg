#!/usr/bin/env python3

# This is a reimplementation of math.rs from the Rust egg repository
# Things not present in this version:
# * custom cost function
# * analysis / guards for rules
#   + this disables some rules and tests
# * the last three tests

from collections import namedtuple
from typing import Any, List

from snake_egg import EGraph, Rewrite, vars

# Operations
Diff = namedtuple("Diff", "x y")
Integral = namedtuple("Integral", "x y")

Add  = namedtuple("Add", "x y")
Sub  = namedtuple("Sub", "x y")
Mul  = namedtuple("Mul", "x y")
Div  = namedtuple("Div", "x y")
Pow  = namedtuple("Pow", "x y")
Ln   = namedtuple("Ln", "x")
Sqrt = namedtuple("Sqrt", "x")

Sin  = namedtuple("Sin", "x")
Cos  = namedtuple("Cos", "x")

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
        if op == Add:
            return a + b
        if op == Sub:
            return a - b
        if op == Mul:
            return a * b
        if op == Div and b != 0.0:
            return a / b
    except:
        pass
    return None


# Rewrite rules, not all are currently used since gaurds aren't in snake-egg yet
a, b, c, x, f, g, y = vars("a b c x f g y") # type: ignore
list_rules: List[List[Any]] = [
  # name,        from,               to
  ["comm-add",   Add(a, b),          Add(b, a)],
  ["comm-mul",   Mul(a, b),          Mul(b, a)],
  ["assoc-add",  Add(a, Add(b, c)),  Add(Add(a, b), c)],
  ["assoc-mul",  Mul(a, Mul(b, c)),  Mul(Mul(a, b), c)],

  ["sub-canon",  Sub(a, b),  Add(a, Mul(-1, b))],
  # rw!("div-canon"; "(/ ?a ?b)" => "(* ?a (pow ?b -1))" if is_not_zero("?b")),
  # // rw!("canon-sub"; "(+ ?a (* -1 ?b))"  => "(- ?a ?b)"),
  # // rw!("canon-div"; "(* ?a (pow ?b -1))" => "(/ ?a ?b)" if is_not_zero("?b")),

  ["zero-add",  Add(a, 0),  a],
  ["zero-mul",  Mul(a, 0),  0],
  ["one-mul",   Mul(a, 1),  a],

  ["add-zero",  a,  Add(a, 0)],
  ["mul-one",   a,  Mul(a, 1)],

  ["cancel-sub",  Sub(a, a),  0],
  # rw!("cancel-div"; "(/ ?a ?a)" => "1" if is_not_zero("?a")),

  ["distribute",  Mul(a, Add(b, c)),          Add(Mul(a, b), Mul(a, c))],
  ["factor",      Add(Mul(a, b), Mul(a, c)),  Mul(a, Add(b, c))],

  ["pow-mul",  Mul(Pow(a, b), Pow(a, c)),  Pow(a, Add(b, c))],
  # rw!("pow0"; "(pow ?x 0)" => "1"
  #            if is_not_zero("?x")),
  ["pow1",     Pow(x, 1),                  x],
  ["pow2",     Pow(x, 2),                  Mul(x, x)],
  # rw!("pow-recip"; "(pow ?x -1)" => "(/ 1 ?x)"
  #            if is_not_zero("?x")),
  # rw!("recip-mul-div"; "(* ?x (/ 1 ?x))" => "1" if is_not_zero("?x")),

  # rw!("d-variable"; "(d ?x ?x)" => "1" if is_sym("?x")),
  # rw!("d-constant"; "(d ?x ?c)" => "0" if is_sym("?x") if is_const_or_distinct_var("?c", "?x")),

  ["d-add",  Diff(x, Add(a, b)),  Add(Diff(x, a), Diff(x, b))],
  ["d-mul",  Diff(x, Mul(a, b)),  Add(Mul(a, Diff(x, b)), Mul(b, Diff(x, a)))],

  ["d-sin",  Diff(x, Sin(x)),  Cos(x)],
  ["d-cos",  Diff(x, Cos(x)),  Mul(-1, Sin(x))],

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

  ["i-one",    Integral(1, x),          x],
  # rw!("i-power-const"; "(i (pow ?x ?c) ?x)" =>
  #            "(/ (pow ?x (+ ?c 1)) (+ ?c 1))" if is_const("?c")),
  ["i-cos",    Integral(Cos(x), x),     Sin(x)],
  ["i-sin",    Integral(Sin(x), x),     Mul(-1, Cos(x))],
  ["i-sum",    Integral(Add(f, g), x),  Add(Integral(f, x), Integral(g, x))],
  ["i-dif",    Integral(Sub(f, g), x),  Sub(Integral(f, x), Integral(g, x))],
  ["i-parts",  Integral(Mul(a, b), x),
               Sub(Mul(a, Integral(b, x)), Integral(Mul(Diff(x, a), Integral(b, x)), x))],
]
# fmt: on

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


def is_equal(expr_a, expr_b, iters=5):
    egraph = EGraph(eval_math)

    id_a = egraph.add(expr_a)
    id_b = egraph.add(expr_b)

    egraph.run(rules, iters)

    return egraph.equiv(id_a, id_b)



def test_math_associate_adds():
    expr_a = Add(1, Add(2, Add(3, Add(4, Add(5, Add(6, 7))))))
    expr_b = Add(7, Add(6, Add(5, Add(4, Add(3, Add(2, 1))))))
    assert is_equal(expr_a, expr_b)

def test_math_simplify_add():
    expr_a = Add(x, Add(x, Add(x, x)))
    expr_b = Mul(4, x)
    assert is_equal(expr_a, expr_b)

def test_math_powers():
    expr_a = Mul(Pow(2, x), Pow(2, y))
    expr_b = Pow(2, Add(x, y))
    assert is_equal(expr_a, expr_b)

def test_math_simplify_const():
    expr_a = Add(1, Sub(a, Mul(Sub(2, 1), a)))
    expr_b = 1
    assert is_equal(expr_a, expr_b)

# def test_math_simplify_root():
#     expr_a = div(1, sub(div(add(1, sqrt(five)), 2),
#                         div(sub(1, sqrt(five)), 2)))
#     expr_b = div(1, sqrt(five))
#     assert is_equal(expr_a, expr_b)

def test_math_simplify_factor():
    expr_a = Mul(Add(x, 3), Add(x, 1))
    expr_b = Add(Add(Mul(x, x), Mul(4, x)), 3)
    assert is_equal(expr_a, expr_b)

# def test_math_diff_same():
#     expr_a = diff(x, x)
#     expr_b = 1
#     assert is_equal(expr_a, expr_b)

# def test_math_diff_different():
#     expr_a = diff(x, y)
#     expr_b = 0
#     assert is_equal(expr_a, expr_b)

# def test_math_diff_simple1():
#     expr_a = diff(x, add(1, mul(2, x)))
#     expr_b = 2
#     assert is_equal(expr_a, expr_b)

# def test_math_diff_simple2():
#     expr_a = diff(x, add(1, mul(y, x)))
#     expr_b = y
#     assert is_equal(expr_a, expr_b)

# def test_math_diff_ln():
#     expr_a = diff(x, ln(x))
#     expr_b = div(1, x)
#     assert is_equal(expr_a, expr_b)

# def test_diff_power_simple():
#     expr_a = diff(x, pow(x, 3))
#     expr_b = mul(3, pow(x, 2))
#     assert is_equal(expr_a, expr_b)

# def test_diff_power_harder():
#     expr_a = diff(x, sub(pow(x, 3), mul(7, pow(x, 2))))
#     expr_b = mul(x, sub(mul(3, x), 14))
#     assert is_equal(expr_a, expr_b)

def test_integ_one():
    expr_a = Integral(1, x)
    expr_b = x
    assert is_equal(expr_a, expr_b)

def test_integ_sin():
    expr_a = Integral(Cos(x), x)
    expr_b = Sin(x)
    assert is_equal(expr_a, expr_b)

# def test_integ_x():
#     expr_a = inte(pow(x, 1), x)
#     expr_b = div(pow(x, 2), 2)
#     assert is_equal(expr_a, expr_b)

# def test_integ_part1():
#     expr_a = inte(mul(x, cos(x)), x)
#     expr_b = add(mul(x, sin(x)), cos(x))
#     assert is_equal(expr_a, expr_b)

# def test_integ_part2():
#     expr_a = inte(mul(cos(x), x), x)
#     expr_b = add(mul(x, sin(x)), cos(x))
#     assert is_equal(expr_a, expr_b)

# def test_integ_part3():
#     expr_a = inte(ln(x), x)
#     expr_b = sub(mul(x, ln(x)), x)
#     assert is_equal(expr_a, expr_b)
