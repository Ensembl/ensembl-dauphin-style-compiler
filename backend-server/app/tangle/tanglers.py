import itertools

from numbers import zigzag, delta
from abstract import AtomicTangling, Tangling, Getter, TanglerTarget, Tangler

#
# FILTERS
#

class AllFiltering:
    def __init__(self, value):
        self._value = value

    def create(self):
        return None

    def check(self, _row, _state):
        return self._value

class ConditionFilter:
    def bid(self, config, name):
        our_config = config.tangles[name]
        if "condition" in our_config:
            return True
        return False
    
    def make(self, _factory, config, name, _input):
        our_config = config.tangles[name]
        key = our_config["condition"]
        return AllFiltering(key in config.conditions)

class FirstFiltering:
    def __init__(self, source):
        self._source = source

    def create(self):
        return set()

    def check(self, row, state):
        value = self._source.get(row)
        out = value not in state
        state.add(value)
        return out

class FirstFilter:
    def bid(self, config, name):
        our_config = config.tangles[name]
        if "first" in our_config:
            return True
        return False

    def make(self, factory, config, name, input):
        our_config = config.tangles[name]
        source = factory.get_source(config,input,our_config["first"])
        return FirstFiltering(source)

#
# ATOMIC
#

class StringTangling(AtomicTangling):
    def __init__(self, our_config):
        super().__init__(our_config,"string",str)

    def finish(self, out, state):
        self._emit_strings(out,'name',state)

class StringTangler(Tangler):
    def __init__(self):
        super().__init__([TanglerTarget(["string"],[],[],[],[])])

    def tangling(self):
        return StringTangling

class NumberTangling(AtomicTangling):
    def __init__(self, our_config):
        super().__init__(our_config,'number',int)
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
        super().__init__([TanglerTarget(["number"],[],[],[("delta",False),("positive",[False])],[])])

    def tangling(self):
        return NumberTangling

#
# CLASSIFIED
#

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
        super().__init__([TanglerTarget(["classify"],[],[],[],["keys","values"])])

    def tangling(self):
        return ClassifiedTangling

#
# INTERVAL
#

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
            TanglerTarget(["start","length"],[],["delta"],[],["starts","lengths"]),
            TanglerTarget(["start","end"],[],["delta"],[],["starts","lengths"])
        ])

    def tangling(self):
        return IntervalTangling
