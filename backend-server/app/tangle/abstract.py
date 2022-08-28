import itertools

from numbers import compress, lesqlite2

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
            out[name] = compress(value)

    def _emit_number(self,out,key,value):
        name = self._config[key]
        if self._uncompressed:
            out[name] = value
        else:
            out[name] = compress(lesqlite2(value))

    def row(self, row, state):
        self._getter.get(row,state)

class Getter:
    def __init__(self, our_config, normal_sources, aux_sources, callback):
        self._our_config = our_config
        self._normal_sources = normal_sources
        self._aux_sources = aux_sources
        self._callback = callback

    def _get_value(self, row, key):
        for source in self._our_config[key]:
            value = source.get(row)
            if value:
                return value
        return None

    def get(self, row, state):
        values = []
        is_list = None
        for (source_key,type_) in self._normal_sources:
            value = self._get_value(row,source_key)
            if is_list is None:
                is_list = (not isinstance(value,type_)) and isinstance(value,list)
            if is_list:
                value = [type_(x) for x in value]
            else:
                value = type_(value)
            values.append(value)
        for (source_key,type_,default) in self._aux_sources:
            if source_key in self._our_config and self._our_config[source_key] is not None:
                value = self._get_value(row,source_key)
            else:
                value = default
            if is_list:
                this_is_list = (not isinstance(value,type_)) and isinstance(value,list)
                if not this_is_list:
                    value = itertools.repeat(value)
            values.append(value)
        if is_list:
            for row in zip(*values):
                self._callback(state,*row)
        else:
            self._callback(state,*values)
        return values

class TanglerTarget:
    def __init__(self,required_sources,required,optional_sources,optional,names):
        self._required_sources = required_sources
        self._required = required
        self._optional_sources = optional_sources
        self._optional = optional
        self._names = names

    def bid(self, config, name):
        our_config = config.tangles[name]
        for key in (self._required + self._required_sources):
            if key not in our_config:
                return None
        return len(self._required) + len(self._required_sources)

    def _make_sources(self, factory, config, input, our_config):
        if not isinstance(our_config,list):
            our_config = [our_config]
        return [factory.get_source(config,input,x) for x in our_config]

    def make(self, factory, config, name, input):
        our_config = config.tangles[name]
        out = {}
        for source in self._required_sources:
            out[source] = self._make_sources(factory,config,input,our_config[source])
        for required in self._required:
            out[required] = our_config[required]
        for source in self._optional_sources:
            if source in our_config:
                out[source] = self._make_sources(factory,config,input,our_config[source])
            else:
                out[source] = []
        for (optional,default) in self._optional:
            if default is not None:
                out[optional] = our_config.get(optional,default)
        out["uncompressed"] = our_config.get("uncompressed",False)
        name = our_config.get("name",name)
        out["name"] = name
        if name != '':
            name += "_"
        for subname in self._names:
            key = subname + "_name"
            if key in our_config:
                out[key] = our_config[key]
            else:
                out[key] = name + subname
        return out

class Tangler:
    def __init__(self,targets):
        self._targets = targets

    def _find(self, config, name):
        for target in self._targets:
            bid = target.bid(config,name)
            if bid is not None:
                return (bid,target)
        return (None,None)

    def bid(self, config, name):
        return self._find(config,name)[0]

    def make(self, factory, config, name, input):
        target = self._find(config,name)[1]
        return (self.tangling())(target.make(factory,config,name,input))

class AtomicTangling(Tangling):
    def __init__(self, our_config, key, ctor):
        super().__init__(our_config,Getter(our_config,[(key,ctor)],[],self._add))

    def create(self):
        return []

    def _add(self, state,value):
        state.append(value)

