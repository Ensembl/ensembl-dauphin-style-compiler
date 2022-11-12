import itertools

from ..abstract.tangler import Tangler
from ..abstract.tangling import Tangling
from ..abstract.getter import Getter
from ..abstract.tangler import TanglerConfigBuilder

class ClassifiedTangling(Tangling):
    def __init__(self, config, our_config):
        super().__init__(our_config,Getter(config,our_config,[("classify",str)],[],self._add))

    def create(self):
        return ({},[],[])

    def _add(self, state, value):
        index = state[0].get(value,-1)
        if index == -1:
            index = len(state[1])
            state[0][value] = index
            state[1].append(value)
        state[2].append(index)

    def finish(self, out, state, run_config):
        self._emit_strings(out,run_config,'keys_name',state[1])
        self._emit_number(out,run_config,'values_name',state[2])

    def finish2(self, out, state, run_config):
        self._emit2(["SYRLZ","SYRAA"],out,run_config,'name',[state[1],state[2]])

class ClassifiedTangler(Tangler):
    def __init__(self):
        super().__init__([TanglerConfigBuilder([
            ("classify",True)
        ],["keys","values"])])

    def tangling(self):
        return ClassifiedTangling
