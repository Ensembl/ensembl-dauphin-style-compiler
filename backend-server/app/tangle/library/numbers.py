from ..util.numbers import zigzag, delta
from ..abstract.tangler import Tangler
from ..abstract.tangling import AtomicTangling
from ..abstract.tangler import TanglerConfigBuilder

class NumberTangling(AtomicTangling):
    def __init__(self, config, our_config):
        super().__init__(config,our_config,'number',int)
        self._delta = our_config["delta"]
        self._positive = our_config["positive"]

    def finish(self, out, state, run_config):
        if self._delta:
            state = delta(state)
        if not self._positive:
            state = zigzag(state)
        self._emit_number(out,run_config,'name',state)

    def finish2(self, out, state, run_config):
        spec = "N"
        if self._delta:
            spec += "D"
        if not self._positive:
            spec += "Z"
        self._emit_number2(spec,out,run_config,'name',state)


class NumberTangler(Tangler):
    def __init__(self):
        super().__init__([TanglerConfigBuilder([
            ("number",True),
            ("delta",False,False),
            ("positive",False,False)
        ],[])])

    def tangling(self):
        return NumberTangling
