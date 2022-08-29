from ..util.numbers import compress, lesqlite2
from .getter import Getter

class Tangling:
    def __init__(self, our_config, getter):
        self._uncompressed = our_config["uncompressed"]
        self._config = our_config
        self._getter = getter

    def _emit_strings(self,out,key,value):
        name = self._config[key]
        if self._uncompressed:
            out[name] = value
        else:
            out[name] = compress("\0".join(value))

    def _emit_number(self,out,key,value):
        name = self._config[key]
        if self._uncompressed:
            out[name] = value
        else:
            out[name] = compress(lesqlite2(value))

    def row(self, row, state):
        self._getter.get(row,state)

class AtomicTangling(Tangling):
    def __init__(self, config, our_config, key, ctor):
        super().__init__(our_config,Getter(config,our_config,[(key,ctor)],[],self._add))

    def create(self):
        return []

    def _add(self, state,value):
        state.append(value)
