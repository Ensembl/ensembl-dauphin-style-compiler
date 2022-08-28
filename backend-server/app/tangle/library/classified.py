import itertools

from abstract.tangler import Tangler
from abstract.tangling import Tangling
from abstract.getter import Getter
from abstract.tangler import TanglerConfigBuilder

class ClassifiedTangling(Tangling):
    def __init__(self, our_config):
        super().__init__(our_config,Getter(our_config,[("classify",str)],[],self._add))

    def create(self):
        return ({},[],[])

    def _add(self, state, value):
        index = state[0].get(value,-1)
        if index == -1:
            index = len(state[1])
            state[0][value] = index
            state[1].append(value)
        state[2].append(index)

    def finish(self, out, state):
        self._emit_strings(out,'keys_name',state[1])
        self._emit_number(out,'values_name',state[2])

class ClassifiedTangler(Tangler):
    def __init__(self):
        super().__init__([TanglerConfigBuilder([
            ("classify",True)
        ],["keys","values"])])

    def tangling(self):
        return ClassifiedTangling
