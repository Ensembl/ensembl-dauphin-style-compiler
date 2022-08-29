import json
import sys, os, toml

from .library.filters import FirstFilter
from .library.atomics import StringTangler, NumberTangler
from .library.count import CountTangler, GroupCountTangler
from .library.classified import ClassifiedTangler
from .library.interval import IntervalTangler
from .library.sources import GetSourceType, AttrSourceType

class TangleException(Exception):
    pass

class Config:
    def __init__(self,config,conditions,processor):
        self.tangles = config.get("tangle",{})
        self.inputs = config.get("input",{})
        self.conditions = conditions
        self.processor = processor
        if "default" not in self.inputs:
            self.inputs["default"] = {}

class SourceHolder:
    def __init__(self, config, source_config, source):
        self._source = source
        complex = False
        self._process = None
        process_name = source_config.get('process',None)
        if process_name is not None:
            self._process = getattr(config.processor,process_name)
            complex = True
        self.row = self._cooked_row if complex else self._raw_row

    def _raw_row(self,row):
        return self._source.get(row)

    def _cooked_row(self,row):
        out = self._raw_row(row)
        if self._process is not None:
            out = self._process(out)
        return out

class TangleFactory:
    def __init__(self):
        self._tanglers = []
        self._filters = []
        self._sources = {}
        self.add_defaults()

    def register_tangler(self, tangler):
        self._tanglers.append(tangler)

    def register_source_type(self, name, source_type):
        self._sources[name] = source_type

    def register_filter(self, filter):
        self._filters.append(filter)

    def add_defaults(self):
        self.register_tangler(StringTangler())
        self.register_tangler(ClassifiedTangler())
        self.register_tangler(IntervalTangler())
        self.register_tangler(NumberTangler())
        self.register_tangler(CountTangler())
        self.register_tangler(GroupCountTangler())
        self.register_source_type("get",GetSourceType())
        self.register_source_type("getattr",AttrSourceType())
        self.register_filter(FirstFilter())

    def _make_filters(self, name, config):
        input_key = config.tangles[name].get("input","default")
        out = []
        for filter in self._filters:
            if filter.bid(config,name):
                out.append(filter.make(self,config,name,input_key))
        return out

    def _make_tangling(self, name, config):
        bid = None
        out = None
        for tangler in self._tanglers:
            this_bid = tangler.bid(config,name)
            if this_bid is not None and (bid is None or this_bid >= bid):
                bid = this_bid
                out = tangler
        input_key = config.tangles[name].get("input","default")
        return out.make(self,config,name,input_key)

    def get_source(self, config, input_key, source_config):
        # sort out args with shortcut values
        if isinstance(source_config,str):
            source_config = { 'field': source_config }
        if input_key is None:
            input_key = 'default'
        # get field from config
        field_key = source_config.get('field',None)
        if field_key is None:
            raise Exception("missing field subkey in source spec")
        # get type from config. If not there explicitly, use default_source_type from input
        type_key = source_config.get('type',None)
        if type_key is None:
            input_config = config.inputs.get(input_key,{})
            type_key = input_config.get("default_source_type")
        if type_key is None:
            raise Exception("missing source type and no default specified for input")
        if type_key not in self._sources:
            raise Exception("No such source type '{}'".format(type_key))
        # create
        values = source_config.copy()
        for key in ('field','type'):
            values.pop(key,None)
        return SourceHolder(config,source_config,self._sources[type_key].make(field_key,values))

    def make_from_config(self, config, conditions=[], processor=None):
        conditions = set(conditions)
        config = Config(config,conditions,processor)
        tanglings = {}
        for name in config.tangles:
            condition = config.tangles[name].get("condition")
            if condition is not None:
                if condition not in conditions:
                    continue
            input_key = config.tangles[name].get("input","default")
            if input_key not in tanglings:
                input = config.inputs.get(input_key,{})
                if "name" not in input:
                    input["name"] = input_key
                tanglings[input_key] = (input,[])
            filters = self._make_filters(name,config)
            tangling = self._make_tangling(name,config)
            if tangling is None:
                raise TangleException("No tangle available for {}".format(name))
            tanglings[input_key][1].append((tangling,filters))
        return Tangle(tanglings.values())

    def make_from_toml(self, config, conditions=[],processor=None):
        return self.make_from_config(toml.loads(config),conditions,processor)

    def make_from_tomlfile(self, path, conditions=[],processor=None):
        with open(path,"r") as f:
            return self.make_from_toml(f.read(),conditions,processor)

class Tangle:
    def __init__(self, tanglings):
        self._tanglings = tanglings
        self._conditions = set()

    def run(self,out,inputs):
        for (input,tanglings) in self._tanglings:
            data = inputs.get(input["name"],[])
            tangle_run = [ (t,[(f,f.create()) for f in f],t.create()) for (t,f) in tanglings ]
            for row in data:
                for (tangle,filters,state) in tangle_run:
                    if not all([f.check(row,s) for (f,s) in filters]):
                        continue
                    tangle.row(row,state)
            for (tangle,_,state) in tangle_run:
                tangle.finish(out,state)
        return out
