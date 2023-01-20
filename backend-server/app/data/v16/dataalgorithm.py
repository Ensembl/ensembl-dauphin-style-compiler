import logging
from typing import Any, ByteString, List
from .numbers import lesqlite2

class NumberSourceAlgorithm:
    def __init__(self, code: List[str]):
        self._code = code.pop()
        if self._code == "A" or self._code == "L":
            pass
        else:
            raise Exception("bad code")

    def make(self, expr: List[Any], value):
        if self._code == "A":
            expr.append(value)
        elif self._code == "L":
            expr.append(lesqlite2(value))

def delta(input: List[int]) -> List[int]:
    value: int = 0
    output: List[int] = []
    for item in input:
        output.append(item - value)
        value = item
    return output

class NumberAlgorithm:
    def __init__(self, code: List[str]):
        self._code = code.pop()
        if self._code == "R":
            self._child = NumberSourceAlgorithm(code)
        elif self._code == "Z" or self._code == "D":
            self._child = NumberAlgorithm(code)
        else:
            raise Exception("bad code")            

    def make(self, expr: List[Any],value):
        if self._code == "R":
            self._child.make(expr,value)
        elif self._code == "Z":
            self._child.make(expr,[ x*2 if x>=0 else -2*x-1 for x in value ])
        elif self._code == "D":
            self._child.make(expr,delta(value))

class StringAlgorithm:
    def __init__(self, code: List[str]):
        self._code = code.pop()           
        if self._code == "A" or self._code == "C" or self._code == "Z":
            pass
        elif self._code == "Y":
            self._index = NumberAlgorithm(code)
            self._values = StringAlgorithm(code)
        else:
            raise Exception("bad code")

    def make(self, expr: List[Any],value):
        if self._code == "A":
            expr.append(value)
        elif self._code == "C":
            expr.append(value.encode("utf8"))
        elif self._code == "Z":
            data = b"\0".join([v.encode("utf8") for v in value])
            if len(value) > 0:
                data = data + b"\0"
            expr.append(data)
        elif self._code == "Y":
            values = list(set(value))
            lookup = { k: i for (i,k) in enumerate(values) }
            indexes = [ lookup[v] for v in value ]
            self._index.make(expr,indexes)
            self._values.make(expr,values)

class BooleanAlgorithm:
    def __init__(self, code: List[str]):
        self._code = code.pop()
        if self._code == "A" or self._code == "B":
            pass
        else:
            raise Exception("bad code")            

    def make(self, expr: List[Any],value):
        if self._code == "A":
            expr.append(value)
        elif self._code == "B":
            output: ByteString = bytearray()
            output += [ 1 if v else 0 for v in value ]
            expr.append(value)

class DataAlgorithm:
    def __init__(self, code: str):
        code = list(code)
        code.reverse()
        self._code = code.pop()
        if self._code == "N":
            self._kid = NumberAlgorithm(code)
        elif self._code == "S":
            self._kid = StringAlgorithm(code)
        elif self._code == "B":
            self._kid = BooleanAlgorithm(code)
        else:
            raise Exception("bad code")

    def make(self, data):
        expr = []
        self._kid.make(expr,data)
        return expr

def data_algorithm(code, data):
    alg = DataAlgorithm(code)
    return [code] + alg.make(data)
