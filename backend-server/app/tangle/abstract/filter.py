class Filter:
    def __init__(self,builder):
        self._builder = builder

    def bid(self, config, name):
        return self._builder.bid(config.tangles[name]) is not None

    def make(self, factory, config, name, input):
        return (self.filtering())(self._builder.make(factory,config,config.tangles[name],input))
