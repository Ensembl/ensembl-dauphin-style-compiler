import json
import sys, os, toml
import unittest

# to allow tests in this file desipte relative imports
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from library.filters import FirstFilter
from library.atomics import StringTangler, NumberTangler
from library.classified import ClassifiedTangler
from library.interval import IntervalTangler
from library.sources import GetSourceType, AttrSourceType

class TangleException(Exception):
    pass

class Config:
    def __init__(self,config,conditions):
        self.tangles = config.get("tangle",{})
        self.inputs = config.get("input",{})
        self.conditions = conditions
        if "default" not in self.inputs:
            self.inputs["default"] = {}

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
        return self._sources[type_key].make(field_key,values)

    def make_from_config(self, config, conditions=[]):
        conditions = set(conditions)
        config = Config(config,conditions)
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

    def make_from_toml(self, config, conditions=[]):
        return self.make_from_config(toml.loads(config),conditions)

class Tangle:
    def __init__(self, tanglings):
        self._tanglings = tanglings
        self._conditions = set()

    def run(self,inputs):
        out = {}
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

def test_data(filename):
    with open(os.path.join(os.path.dirname(__file__),"testdata",filename),"r") as f:
        return f.read()

class TangleTestCase(unittest.TestCase):
    def test_smoke(self):
        tangle_factory = TangleFactory()
        test_config = test_data("smoke-config.toml")
        data_in = json.loads(test_data("smoke-in.json"))
        data_out = json.loads(test_data("smoke-out.json"))
        tangle = tangle_factory.make_from_toml(test_config,["on"])
        out = tangle.run(data_in)
        self.assertEqual(out,data_out)

if __name__ == '__main__':
    unittest.main()
