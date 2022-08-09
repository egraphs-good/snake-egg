from ._internal import PyEGraph
from ._internal import PyId as Id
from ._internal import PyPattern as Pattern
from ._internal import PyRewrite as Rewrite
from ._internal import PyVar as Var
from ._internal import vars


class EGraph(PyEGraph):

    def extract(self, expr):
        result = super().extract(expr)
        if len(result) == 1:
            return result[0]
        else:
            return result
