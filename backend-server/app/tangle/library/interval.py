import math

from ..util.numbers import zigzag, delta
from ..abstract.tangler import Tangler
from ..abstract.tangling import Tangling
from ..abstract.getter import Getter
from ..abstract.tangler import TanglerConfigBuilder

class IntervalTangling(Tangling):
    def __init__(self, config,our_config):
        other_key = "end" if 'end' in our_config else "length"
        self._other_is_end = other_key == "end"
        super().__init__(our_config,Getter(config,our_config,[('start',int),(other_key,int)],[("delta",int,0)],self._add))
        if our_config["process"] is not None:
            self._process = getattr(config.processor,our_config['process'])
        else:
            self._process = None

    def create(self):
        return ([],[])

    def _add(self, state, start_value, other_value, delta_value):
        start = start_value
        length = other_value-start_value if self._other_is_end else other_value
        start += delta_value if delta_value else 0
        if self._process is not None:
            (start,length) = self._process(start,length)
        state[0].append(start)
        state[1].append(length)

    def finish(self, out, state, run_config):
        self._emit_number(out,run_config,'starts_name',zigzag(delta(state[0])))
        self._emit_number(out,run_config,'lengths_name',zigzag(delta(state[1])))

    def finish2(self, out, state, run_config):
        self._emit_number2("NDZ",out,run_config,'starts_name',state[0])
        self._emit_number2("NDZ",out,run_config,'lengths_name',state[1])

class IntervalTangler(Tangler):
    def __init__(self):
        super().__init__([
            TanglerConfigBuilder([
                ("start",True),
                ("length",True),
                ("delta",True,None),
                ('process',False,None),
            ],["starts","lengths"]),
            TanglerConfigBuilder([
                ("start",True),
                ("end",True),
                ("delta",True,None),                
                ('process',False,None),
                ("store_end",False,False)
            ],["starts","lengths"])
        ])

    def tangling(self):
        return IntervalTangling
