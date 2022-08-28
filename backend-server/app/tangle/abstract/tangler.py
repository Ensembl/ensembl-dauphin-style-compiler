from abstract.configbuilder import ConfigBuilder

class TanglerConfigBuilder(ConfigBuilder):
    def __init__(self,fields,names):
        self._names = names
        super().__init__(fields)

    def make(self, factory, config, name, input):
        our_config = config.tangles[name]
        out = super().make(factory,config,our_config,input)
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

    def bid(self, config, name):
        return super().bid(config.tangles[name])

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
