from typing import Any, Tuple
from model.serialutil import build_map, immute, increase, remute, immute_key
import toml

class ProgramSetting:
    def __init__(self, name, data):
        self._name = name
        self._default = data.get("default",False)

    def _collect(self):
        return (self._name,immute(self._default))

class ProgramSpec:
    def __init__(self, in_bundle_name, toml_path):
        with open(toml_path) as f:
            toml_file = toml.loads(f.read())
        self._set = toml_file.get("set","")
        self._in_bundle_name = in_bundle_name
        self._name = toml_file["name"]
        self._version = int(toml_file.get("version","0"))
        self._settings = [ ProgramSetting(name,data) for (name,data) in toml_file.get("setting",{}).items() ]

    def _collect(self):
        names = set([self._in_bundle_name,self._name,self._set])
        keys = set()
        values = set()
        for setting in self._settings:
            (another_key,another_value) = setting._collect()
            keys.add(another_key)
            values.add(another_value)
        return (names,keys,values)

    def _dump_for_wire(self, dumper) -> Any:
        settings = sorted(self._settings, key = lambda x: x._name)
        keys = [s._name for s in settings]
        defaults = [immute(s._default) for s in settings]
        return {
            'in_bundle_name': dumper.name_mapping[self._in_bundle_name],
            'set': dumper.name_mapping[self._set],
            'name': dumper.name_mapping[self._name],
            'version': self._version,
            'keys': increase([dumper.key_mapping[x] for x in keys]),
            'defaults': increase([dumper.value_mapping[x] for x in defaults]),
        }

    def full_name(self) -> Tuple[str,str,int]:
        return (self._set,self._name,self._version)

class AllProgramSpecs:
    def __init__(self):
        self._specs = []

    def add(self, spec: ProgramSpec):
        self._specs.append(spec)

    def _collect(self):
        names = set()
        keys = set()
        values = set()
        for spec in self._specs:
            (more_names,more_keys,more_values) = spec._collect()
            names |= more_names
            keys |= more_keys
            values |= more_values
        return (names,keys,values)

    def serialize(self) -> Any:
        return ProgramsDump(self).data

def rotate(data, key):
    out = {}
    for item in sorted(data, key=key):
        for (key,value) in item.items():
            if key not in out:
                out[key] = []
            out[key].append(value)
    return out

class ProgramsDump:
    def __init__(self, specs: AllProgramSpecs):
        if len(specs._specs) == 0:
            self.data = None
            return
        (names,keys,values) = specs._collect()
        (name_list,self.name_mapping) = build_map(sorted(names))
        (key_list,self.key_mapping) = build_map(sorted(keys))
        (value_list,self.value_mapping) = build_map(sorted(values, key = immute_key()))
        data = []
        for spec in specs._specs:
            data.append(spec._dump_for_wire(self))
        self.data = rotate(data,lambda x: x['name'])
        self.data["name_idx"] = name_list
        self.data["key_idx"] = key_list
        self.data["value_idx"] = [remute(v) for v in value_list]
        for key in [
            "name", "in_bundle_name", "set", "version", "keys", "defaults"
        ]:
            if key not in self.data:
                self.data[key] = []
        for key in [
            "name", "in_bundle_name", "set", "version"
        ]:
            self.data[key] = increase(self.data[key])
