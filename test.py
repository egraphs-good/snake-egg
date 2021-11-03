import unittest
import snake_egg

from collections import namedtuple

Add = namedtuple('Add', 'x y')
Mul = namedtuple('Mul', 'x y')


class TestEGraph(unittest.TestCase):

    def test_simple(self):
        egraph = snake_egg.EGraph()
        a123 = egraph.add(Add(1, Add(2, 3)))
        egraph.add(Add(a123, 4))
        print(str(a123))

        self.assertEqual(egraph.add(Add(7, 8)), egraph.add(Add(7, 8)))
        self.assertTrue(egraph.union(2, Add(1, 1)))


if __name__ == '__main__':
    unittest.main()
