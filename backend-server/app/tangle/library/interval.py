import itertools

from util.numbers import zigzag, delta
from abstract.tangler import Tangler
from abstract.tangling import Tangling
from abstract.getter import Getter
from abstract.tangler import TanglerConfigBuilder

class IntervalTangling(Tangling):
    def __init__(self, our_config):
        other_key = "end" if 'end' in our_config else "length"
        self._other_is_end = other_key == "end"
        super().__init__(our_config,Getter(our_config,[('start',int),(other_key,int)],[("delta",int,0)],self._add))

    def create(self):
        return ([],[])

    def _add(self, state, start_value, other_value, delta_value):
        length = other_value-start_value if self._other_is_end else other_value
        start_value += delta_value if delta_value else 0
        state[0].append(start_value)
        state[1].append(length)

    def finish(self, out, state):
        self._emit_number(out,'starts_name',zigzag(delta(state[0])))
        self._emit_number(out,'lengths_name',zigzag(delta(state[1])))

class IntervalTangler(Tangler):
    def __init__(self):
        super().__init__([
            TanglerConfigBuilder([
                ("start",True),
                ("length",True),
                ("delta",True,None),                
            ],["starts","lengths"]),
            TanglerConfigBuilder([
                ("start",True),
                ("end",True),
                ("delta",True,None),                
            ],["starts","lengths"])
        ])

    def tangling(self):
        return IntervalTangling
