import itertools

from ..abstract.configbuilder import ConfigBuilder
from ..abstract.filter import Filter

class FirstFiltering:
    def __init__(self, our_config):
        self._source = our_config["first"]

    def create(self):
        return set()

    def check(self, row, state):
        values = [x.row(row) for x in self._source]
        new = all([x not in state for x in values])
        state |= set(values)
        return new

class FirstFilter(Filter):
    def __init__(self):
        super().__init__(ConfigBuilder([
            ("first",True)
        ]))

    def filtering(self):
        return FirstFiltering
