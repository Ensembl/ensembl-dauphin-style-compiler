import logging
from typing import Set
import cbor2

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

def _build_map(data):
    data = sorted(data)
    mapping = { v: i for (i,v) in enumerate(data) }
    return (data,mapping)

def increase(data):
    prev = 0
    out = []
    for item in data:
        out.append(item-prev)
        prev = item
    return out

class Track:
    def __init__(self,name):
        self._name = name
        self._program = name
        self._scales = None
        self._triggers = []
        self._extra = []
        self._tags = []
        self._set = []

    def ingest_toml(self,data):
        if "program" in data:
            self._program = data["program"]
        if "scales" in data:
            self._scales = [int(x) for x in data["scales"]]
        if "triggers" in data:
            self._triggers += [tuple(x) for x in data["triggers"]]
        if "extra" in data:
            self._extra += [tuple(x) for x in data["extra"]]
        if "tags" in data:
            self._tags += data["tags"]
        if "set" in data:
            self._set += [tuple(x) for x in data["set"]]

    def _collect(self) -> Set:
        switches = set()
        switches |= set(self._triggers)
        switches |= set(self._extra)
        switches |= set(self._set)
        return (switches,set((self._program,)),set(self._tags))

    def _dump_for_wire(self, dumper, name):
        return {
            "name": name,
            "program": dumper.program_mapping[self._program],
            "scales": self._scales,
            "tags": [dumper.tag_mapping[x] for x in self._tags],
            "triggers": increase(sorted([dumper.switch_mapping[x] for x in self._triggers])),
            "extra": increase(sorted([dumper.switch_mapping[x] for x in self._extra])),
            "set": increase(sorted([dumper.switch_mapping[x] for x in self._set])),
        }

class Expansion:
    def __init__(self,name):
        self._name = name
        self._channel = None
        self._triggers = []

    def ingest_toml(self,data):
        if "name" in data:
            self._name = data["name"]
        if "channel" in data:
            self._channel = tuple(data["channel"])
        if "triggers" in data:
            self._triggers += [tuple(x) for x in data["triggers"]]

    def _collect(self) -> Set:
        return (set(self._triggers),set([self._channel]))

    def _dump_for_wire(self, dumper):
        return {
            "e-name": self._name,
            "e-channel": dumper.channel_mapping[self._channel],
            "e-triggers": increase(sorted([dumper.switch_mapping[x] for x in self._triggers]))
        }

class Tracks:
    def __init__(self,expanded_toml=None):
        self._tracks = {}
        self._expansions = {}
        if expanded_toml is not None:
            self.ingest_toml(expanded_toml)

    def ingest_toml(self, data):
        includes = {}
        includes_data = data.get("include",None)
        if includes_data is not None:
            for (name,value) in includes_data.items():
                includes[name] = value
        for (name,track_data) in data.get("track",{}).items():
            track = Track(name)
            for inc_name in track_data.get("include",[]):
                track.ingest_toml(includes[inc_name])
            track.ingest_toml(track_data)
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
        for track in self._tracks.values():
            (more_switches,more_programs,more_tags) = track._collect()
            switches |= more_switches
            programs |= more_programs
            tags |= more_tags
        for expansion in self._expansions.values():
            (more_switches,more_channels) = expansion._collect()
            switches |= more_switches
            channels |= more_channels
        return (switches,programs,tags,channels)

    def dump_for_wire(self):
        return TracksDump(self).data

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
        (switches,programs,tags,channels) = tracks._collect()
        (channels_idx,self.channel_mapping) = _prefix_encode(channels)
        (switch_tree,self.switch_mapping) = _prefix_encode(switches)
        (program_list,self.program_mapping) = _build_map(programs)
        (tag_list,self.tag_mapping) = _build_map(tags)
        data = {}
        for (name,track) in tracks._tracks.items():
            data[name] = track._dump_for_wire(self,name)
        self.data = rotate(data,lambda x: x[1]['scales'][0])
        (scale_start,scale_end,scale_step) = split_scale(self.data["scales"])
        self.data['scale_start'] = scale_start
        self.data['scale_end'] = scale_end
        self.data['scale_step'] = scale_step
        self.data.pop('scales',None)
        expansions = {}
        for (name,expansion) in tracks._expansions.items():
            expansions[name] = expansion._dump_for_wire(self)
        self.data.update(rotate(expansions,lambda x: x[1]['e-channel']))
        self.data['switch_idx'] = switch_tree
        self.data['program_idx'] = program_list
        self.data['tag_idx'] = tag_list
        self.data['channel_idx'] = channels_idx
