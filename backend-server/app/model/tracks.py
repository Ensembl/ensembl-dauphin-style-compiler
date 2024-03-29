import logging, os.path
from typing import Optional
from model.serialutil import build_map, immute, increase, remute
import cbor2, toml

def _count_prefix(a,b):
    minlen = min(len(a),len(b))
    for i in range(0,minlen):
        if a[i] != b[i]:
            return i
    return minlen

def _prefix_encode(switches):
    tree = []
    mapping = {}
    prev_prefix_len = 0
    prev_switch = []
    for (i,switch) in enumerate(sorted(switches)):
        prefix_len = _count_prefix(prev_switch,switch)
        tree.append([prefix_len-prev_prefix_len,switch[prefix_len:]])
        mapping[switch] = i
        prev_prefix_len = prefix_len
        prev_switch = switch
    return (tree,mapping)

class Track:
    def __init__(self,name,program_name="",scales=None,program_group = "",program_version = 0):
        self._name = name
        self._program_name = program_name
        self._program_set = program_group
        self._program_version = program_version
        self._scales = scales
        self._triggers = []
        self._tags = ""
        self._values = []
        self._settings = []

    def add_trigger(self, path: list[str]):
        self._triggers.append(tuple(path))

    def add_tag(self, tag: str):
        self._tags.append(tag)

    def add_value(self, name: str, value):
        self._values.append((name,immute(value)))

    def add_setting(self, name: str, setting: list[str]):
        self._settings.append((name, tuple(setting)))

    def ingest_toml(self,data,includes):
        if "include" in data:
            for inc_name in data["include"]:
                self.ingest_toml(includes[inc_name],includes)
        if "general" in data:
            self.ingest_toml(data["general"],includes)
        if "program_name" in data:
            self._program_name = data["program_name"]
        if "program_set" in data:
            self._program_set = data["program_set"]
        if "program_version" in data:
            self._program_version = int(data["program_version"])
        if "scales" in data:
            self._scales = [int(x) for x in data["scales"]]
        if "triggers" in data:
            self._triggers += [tuple(x) for x in data["triggers"]]
        if "tags" in data:
            self._tags =  data["tags"]
        if "values" in data:
            for (name,value) in data["values"].items():
                self.add_value(name,value)
        if "settings" in data:
            for (name,setting) in data["settings"].items():
                self._settings.append((name,tuple(setting)))

    def _collect(self):
        switches = set()
        switches |= set(self._triggers)
        switches |= set([x[1] for x in self._settings])
        values = set([x[1] for x in self._values])
        keys = set([x[0] for x in self._settings])
        keys |= set([x[0] for x in self._values])
        return (switches,set([self._program_name,self._program_set]),set([self._tags]),values,keys)

    def _dump_for_wire(self, dumper):
        settings = sorted(self._settings, key = lambda x: x[0])
        values = sorted(self._values, key = lambda x: x[0])
        return {
            "program_name": dumper.program_mapping[self._program_name],
            "program_set": dumper.program_mapping[self._program_set],
            "program_version": self._program_version,
            "scales": self._scales,
            "tags": dumper.tag_mapping[self._tags],
            "triggers": increase(sorted([dumper.switch_mapping[x] for x in self._triggers])),
            "values-keys": increase([dumper.key_mapping[x[0]] for x in values]),
            "values-values": [dumper.value_mapping[x[1]] for x in values],
            "settings-keys": increase([dumper.key_mapping[x[0]] for x in settings]),
            "settings-values": [dumper.switch_mapping[x[1]] for x in settings],
        }

class Expansion:
    def __init__(self,name):
        self._name = name
        self._channel = None
        self._triggers = []
        self._run = None

    def ingest_toml(self,data):
        if "name" in data:
            self._name = data["name"]
        if "channel" in data:
            self._channel = tuple(data["channel"])
        if "triggers" in data:
            self._triggers += [tuple(x) for x in data["triggers"]]
        if "run" in data:
            self._run = data["run"]

    def callback(self):
        return self._run

    def _collect(self) -> tuple[set[str], set[str|None]]:
        return (set(self._triggers),set([self._channel]))

    def _dump_for_wire(self, dumper):
        return {
            "e-name": self._name,
            "e-channel": dumper.channel_mapping[self._channel],
            "e-triggers": increase(sorted([dumper.switch_mapping[x] for x in self._triggers]))
        }

class Tracks:
    def __init__(self,path=None):
        self._tracks = {}
        self._expansions = {}
        self._path = path
        self._includes = {}
        self._cooked = []
        if path is not None:
            self.ingest_toml({
                'include_files': [path]
            })

    def add_track(self,name,track):
        self._tracks[name] = track

    def add_cookeds(self, cooked):
        self._cooked += cooked

    def ingest_toml(self, data, seen = set()):
        if "include_files" in data and self._path is not None:
            for filename in data["include_files"]:
                file_path = os.path.join(os.path.dirname(self._path),filename)
                if file_path in seen:
                    raise Exception("loop in include_files")
                tracks_toml = toml.load(file_path)
                self.ingest_toml(tracks_toml,seen|set([file_path]))
        includes_data = data.get("include",None)
        if includes_data is not None:
            for (name,value) in includes_data.items():
                self._includes[name] = value
        for (name,track_data) in data.get("track",{}).items():
            track = Track(name,program_name=name)
            track.ingest_toml(track_data,self._includes)
            self._tracks[name] = track
        for (name,expansion_data) in data.get("expansion",{}).items():
            expansion = Expansion(name)
            expansion.ingest_toml(expansion_data)
            self._expansions[name] = expansion

    def merge(self, other):
        # later additions get priority (ie local over remote)
        self._tracks.update(other._tracks)
        self._expansions.update(other._expansions)

    def _collect(self):
        switches = set()
        programs = set()
        tags = set()
        channels = set()
        values = set()
        keys = set()
        for track in self._tracks.values():
            (more_switches,more_programs,more_tags,more_values,more_keys) = track._collect()
            switches |= more_switches
            programs |= more_programs
            tags |= more_tags
            values |= more_values
            keys |= more_keys
        for expansion in self._expansions.values():
            (more_switches,more_channels) = expansion._collect()
            switches |= more_switches
            channels |= more_channels
        return (switches,programs,tags,channels,values,keys)

    def dump_for_wire(self):
        return [ x for x in [TracksDump(self).data] + self._cooked if x is not None ]

    def get_expansion(self, name) -> Optional[Expansion]:
        return self._expansions.get(name)

def rotate(data, key):
    out = {}
    for (name,item) in sorted(data.items(), key=key):
        for (key,value) in item.items():
            if key not in out:
                out[key] = []
            out[key].append(value)
    return out

def split_scale(data):
    out = [[],[],[]]
    for item in data:
        out[0].append(item[0])
        out[1].append(item[1])
        out[2].append(item[2])
    return out

class TracksDump:
    def __init__(self, tracks: Tracks):
        if len(tracks._tracks) == 0:
            self.data = None
            return
        (switches,programs,tags,channels,values,keys) = tracks._collect()
        (channels_idx,self.channel_mapping) = _prefix_encode(channels)
        (switch_tree,self.switch_mapping) = _prefix_encode(switches)
        (program_list,self.program_mapping) = build_map(sorted(programs))
        (key_list,self.key_mapping) = build_map(sorted(keys))
        (tag_list,self.tag_mapping) = build_map(sorted(tags))
        (value_list,self.value_mapping) = build_map(values)
        data = {}
        for (name,track) in tracks._tracks.items():
            data[name] = track._dump_for_wire(self)
        self.data = rotate(data,lambda x: x[1]['scales'][0])
        (scale_start,scale_end,scale_step) = split_scale(self.data["scales"])
        self.data['scale_start'] = scale_start
        self.data['scale_end'] = scale_end
        self.data['scale_step'] = scale_step
        self.data["program_version"] = increase(self.data["program_version"])
        self.data.pop('scales',None)
        expansions = {}
        for (name,expansion) in tracks._expansions.items():
            expansions[name] = expansion._dump_for_wire(self)
        self.data.update(rotate(expansions,lambda x: x[1]['e-channel']))
        self.data['switch_idx'] = switch_tree
        self.data['program_idx'] = program_list
        self.data['tag_idx'] = tag_list
        self.data['key_idx'] = key_list
        self.data['channel_idx'] = channels_idx
        self.data['value_idx'] = [remute(x) for x in value_list]
        for key in [
                    "program_name", "program_set", "program_version", "scales",
                    "tags", "triggers", "set", "values", "e-name", "e-channel",
                    "e-triggers", "values-keys", "values-values", "settings-keys",
                    "settings-values"
                ]:
            if key not in self.data:
                self.data[key] = []
