class AttrSource:
    def __init__(self, key):
        self._key = key

    def get(self, row):
        return getattr(row,self._key)

class AttrSourceType:
    def make(self, key, _conf):
        return AttrSource(key)

class GetSource:
    def __init__(self, key):
        self._key = key

    def get(self, row):
        return row.get(self._key,None)

class GetSourceType:
    def make(self, key, _conf):
        return GetSource(key)
