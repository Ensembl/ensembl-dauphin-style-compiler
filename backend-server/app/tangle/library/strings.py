from ..abstract.getter import Getter
from ..abstract.tangler import Tangler
from ..abstract.tangling import AtomicTangling, Tangling
from ..abstract.tangler import TanglerConfigBuilder

class SimpleStringTangling(AtomicTangling):
    def __init__(self, config, our_config):
        super().__init__(config,our_config,"string",str)

    def finish(self, out, state, run_config):
        self._emit_strings(out,run_config,'name',state)

    def finish2(self, out, state, run_config):
        self._emit_strings2("S",out,run_config,'name',state)

class SimpleStringTangler(Tangler):
    def __init__(self):
        super().__init__([TanglerConfigBuilder([
            ("string",True)
        ],[])])

    def tangling(self):
        return SimpleStringTangling

class ComplexStringTangling(Tangling):
    def __init__(self, config, our_config):
        super().__init__(our_config,Getter(config,our_config,[
            ("string",str)
        ],[
            ("without_prefix_source",str,None),
            ("without_suffix_source",str,None),
        ],self._add))
        self._without_prefix = our_config.get('without_prefix',None)
        self._without_suffix = our_config.get('without_suffix',None)

    def create(self):
        return []

    def _add(self, state, string, without_prefix_source, without_suffix_source):
        if without_prefix_source is not None and string.startswith(without_prefix_source):
            string = string[len(without_prefix_source):]
        if without_suffix_source is not None and string.endswith(without_suffix_source) and len(without_suffix_source) > 0:
            string = string[:-len(without_suffix_source)]
        if self._without_prefix is not None and string.startswith(self._without_prefix):
            string = string[len(self._without_prefix):]
        if self._without_suffix is not None and string.endswith(self._without_suffix) and len(self._without_suffix) > 0:
            string = string[:-len(self._without_suffix)]
        state.append(string)

    def finish(self, out, state, run_config):
        self._emit_strings(out,run_config,'name',state)

    def finish2(self, out, state, run_config):
        self._emit_strings2("S",out,run_config,'name',state)

class ComplexStringTangler(Tangler):
    def __init__(self):
        super().__init__([TanglerConfigBuilder([
            ("string",True),
            ("without_prefix_source",True,None),
            ("without_suffix_source",True,None),
            ("without_prefix",False,None),
            ("without_suffix",False,None)
        ],[])])

    def tangling(self):
        return ComplexStringTangling
