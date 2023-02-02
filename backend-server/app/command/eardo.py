import logging
import os
import cbor2

class EardProgram:
    def __init__(self, group, program, version):
        pass

class EardoFile:
    def __init__(self,name,path):
        self._programs = set()
        self._mtime = 0
        self._name = name
        self._path = path
        self.load()

    def name(self):
        return self._name

    def load(self):
        with open(self._path,'rb') as f:
            self._raw = f.read()
            self._data = cbor2.loads(self._raw)
            for program in self._data:
                name = program["metadata"]["name"]
                self._programs.add((name[0],name[1],name[2]))

    def program_names(self):
        return self._programs

    def reload_if_necessary(self):
        if self.reload_necessary():
            logging.info("reloading of '{}' necessary".format(self._path))
            self.load()

    def reload_necessary(self):
        new_mtime = os.stat(self._path).st_mtime
        if self._mtime != new_mtime:
            self._mtime = new_mtime
            return True
        else:
            return False

    def serialise(self):
        return [self._name,self._raw]
