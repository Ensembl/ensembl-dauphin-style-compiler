from ..util.numbers import zigzag, delta
from ..abstract.getter import Getter
from ..abstract.tangler import Tangler
from ..abstract.tangling import Tangling
from ..abstract.tangler import TanglerConfigBuilder

class CountTangling(Tangling):
    def __init__(self, config, our_config):
        super().__init__(our_config,Getter(config,our_config,[('count',list)],[],self._add,collapse_lists=False))

    def create(self):
        return []

    def _add(self, state,value):
        state.append(len(value))

    def finish(self, out, state):
        state = zigzag(delta(state))
        self._emit_number(out,'name',state)

class CountTangler(Tangler):
    def __init__(self):
        super().__init__([TanglerConfigBuilder([
            ("count",True)
        ],[])])

    def tangling(self):
        return CountTangling

class GroupCountTangling(Tangling):
    def __init__(self, config, our_config):
        super().__init__(our_config,Getter(config,our_config,[('group_count',str)],[],self._add))

    def create(self):
        return ({},[])

    def _add(self, state, value):
        if value not in state[0]:
            state[0][value] = 0
            state[1].append(value)
        state[0][value] += 1

    def finish(self, out, state):
        values = [ state[0][x] for x in state[1] ]
        state = zigzag(delta(values))
        self._emit_number(out,'name',state)

class GroupCountTangler(Tangler):
    def __init__(self):
        super().__init__([TanglerConfigBuilder([
            ("group_count",True)
        ],[])])

    def tangling(self):
        return GroupCountTangling
