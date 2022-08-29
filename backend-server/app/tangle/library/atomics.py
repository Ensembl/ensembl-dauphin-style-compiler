from ..util.numbers import zigzag, delta
from ..abstract.getter import Getter
from ..abstract.tangler import Tangler
from ..abstract.tangling import AtomicTangling
from ..abstract.tangler import TanglerConfigBuilder

class StringTangling(AtomicTangling):
    def __init__(self, config, our_config):
        super().__init__(config,our_config,"string",str)

    def finish(self, out, state):
        self._emit_strings(out,'name',state)

class StringTangler(Tangler):
    def __init__(self):
        super().__init__([TanglerConfigBuilder([
            ("string",True)
        ],[])])

    def tangling(self):
        return StringTangling

class NumberTangling(AtomicTangling):
    def __init__(self, config, our_config):
        super().__init__(config,our_config,'number',int)
        self._delta = our_config["delta"]
        self._positive = our_config["positive"]

    def finish(self, out, state):
        if self._delta:
            state = delta(state)
        if not self._positive:
            state = zigzag(state)
        self._emit_number(out,'name',state)

class NumberTangler(Tangler):
    def __init__(self):
        super().__init__([TanglerConfigBuilder([
            ("number",True),
            ("delta",False,False),
            ("positive",False,False)
        ],[])])

    def tangling(self):
        return NumberTangling
