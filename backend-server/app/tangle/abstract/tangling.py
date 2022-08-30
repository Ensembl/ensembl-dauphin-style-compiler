from ..util.numbers import compress, lesqlite2
from .getter import Getter

class Tangling:
    def __init__(self, our_config, getter):
        self._config = our_config
        self._getter = getter

    def _emit_strings(self,out,run_config,key,value):
        name = self._config[key]
        if run_config.to_bytes:
            value = "\0".join(value)
        if run_config.compress:
            value = value.encode("utf8")
        out[name] = value

    def _emit_number(self,out,run_config,key,value):
        name = self._config[key]
        if run_config.to_bytes:
            value = lesqlite2(value)
        if run_config.compress:
            #pass
            value = bytes(value)
        out[name] = value

    def row(self, row, state, _run_config):
        self._getter.get(row,state)

class AtomicTangling(Tangling):
    def __init__(self, config, our_config, key, ctor):
        super().__init__(our_config,Getter(config,our_config,[(key,ctor)],[],self._add))

    def create(self):
        return []

    def _add(self, state,value):
        state.append(value)
