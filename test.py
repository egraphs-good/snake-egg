import inspect
import unittest
# import doctest

from snake_egg import EGraph, Rewrite, Var, vars

from collections import namedtuple
Add = namedtuple('Add', 'x y')
Mul = namedtuple('Mul', 'x y')

x, y, z = vars('x y z')

print(str(Add(x, y)))

rules = [
    Rewrite(lhs=Add(x, y), rhs=Add(y, x), name='add_comm'),
    Rewrite(Mul(x, y),         Mul(y, x)),
    Rewrite(Add(x, Add(y, z)), Add(Add(x, y), z)),
    Rewrite(Mul(x, Mul(y, z)), Mul(Mul(x, y), z)),
    Rewrite(Add(x, 0),         x),
    Rewrite(Mul(x, 0),         0),
    Rewrite(Mul(x, 1),         x),
    Rewrite(Add(x, x),         Mul(x, 2)),
]

for r in rules:
    print(r.name)


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

        add = egraph.add(Add(1, 0))
        egraph.run(rules, iter_limit=2)

        self.assertTrue(egraph.equiv(add, 1))
        self.assertEqual(egraph.add(Add(7, 8)), egraph.add(Add(7, 8)))
        self.assertTrue(egraph.union(2, Add(1, 1), Add(2, 0)))

        # extract two separately
        self.assertEqual(egraph.extract(add), egraph.extract(1))
        # extract two at same time
        a, b = egraph.extract(add, 1)
        self.assertEqual(a, b)


if __name__ == '__main__':
    # import snake_egg
    # print("--- doc tests ---")
    # failed, tested = doctest.testmod(snake_egg, verbose=True, report=True)
    # if failed > 0:
    #     exit(1)
    print("\n\n--- unit tests ---")
    unittest.main(verbosity=2)
