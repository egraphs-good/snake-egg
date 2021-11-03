from typing import Any, Tuple
import unittest


from collections import namedtuple
from snake_egg import EGraph, Rewrite, Var, vars

Add = namedtuple('Add', 'x y')
Mul = namedtuple('Mul', 'x y')

x, y = vars('x y')

rules = [
    Rewrite(Add(x, x), to=Mul(x, 2))
]


egraph = EGraph()

add = egraph.add(Add(1, 1))
mul = egraph.add(Mul(1, 2))

egraph.run(rules, iter_limit=1)
assert egraph.equiv(add, mul)


class TestEGraph(unittest.TestCase):

    def test_vars(self):
        self.assertEqual(x, Var('x'))
        self.assertEqual(str(x), '?x')
        self.assertEqual(repr(x), 'Var(?x)')

    def test_simple(self):
        egraph = EGraph()

        add = egraph.add(Add(1, 1))
        mul = egraph.add(Mul(1, 2))

        egraph.run(rules, iter_limit=1)

        self.assertTrue(egraph.equiv(add, mul))

        self.assertEqual(egraph.add(Add(7, 8)), egraph.add(Add(7, 8)))
        self.assertTrue(egraph.union(2, Add(1, 1), Add(2, 0)))


if __name__ == '__main__':
    unittest.main()
