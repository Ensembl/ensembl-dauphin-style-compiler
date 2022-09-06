def make_sources(factory, config, input, our_config):
    if not isinstance(our_config,list):
        our_config = [our_config]
    return [factory.get_source(config,input,x) for x in our_config]

class ConfigBuilder:
    def __init__(self,fields):
        self._fields = []
        for field in fields:
            if len(field) > 2:
                self._fields.append((field[0],field[1],True,field[2]))
            else:
                self._fields.append((field[0],field[1],False,None))

    def make(self, factory, config, our_config, input):
        out = {}
        for (key,is_source,_,default) in self._fields:
            if is_source:
                if key in our_config:
                    out[key] = make_sources(factory,config,input,our_config[key])
                else:
                    out[key] = []
            else:
                out[key] = our_config.get(key,default)
        return out

    def bid(self, our_config):
        count = 0
        for (key,_,is_optional,_) in self._fields:
            if not is_optional:
                if key not in our_config:
                    return None
            if key in our_config:
                count += 1
        return count
