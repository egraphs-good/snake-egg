from ._internal import PyEGraph  # type: ignore
from ._internal import vars  # type: ignore
from ._internal import PyId as Id  # type: ignore
from ._internal import PyPattern as Pattern  # type: ignore
from ._internal import PyRewrite as Rewrite  # type: ignore
from ._internal import PyVar as Var  # type: ignore


class EGraph(PyEGraph):
    def extract(self, expr):
        result = super().extract(expr)
        if len(result) == 1:
            return result[0]
        else:
            return result
