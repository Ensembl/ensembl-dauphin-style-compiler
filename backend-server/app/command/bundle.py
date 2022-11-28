from __future__ import annotations
import logging, os.path, cbor2
from model.programs import AllProgramSpecs, ProgramSpec
from typing import Any
from core.config import EGS_FILES

class Bundle:
    def __init__(self, name: str, program_path: str, egs_version: int, name_map = None):
        self.name = name
        self._specs = AllProgramSpecs()
        self.path = program_path
        self._program = self.load_program(program_path,egs_version)
        self._egs_version = egs_version
        self._name_map = name_map

    def load_program(self, program_path: str, egs_version: int) -> Any:
        with open(program_path,'rb') as f:
            if egs_version < 15:
                return cbor2.loads(f.read())
            else:
                return f.read()

    def add_program(self, name: str, spec_path: str) -> ProgramSpec:
        if self._egs_version >= 15:
            if not os.path.exists(spec_path):
                raise Exception("missing spec file {}".format(spec_path))
            out = ProgramSpec(name,spec_path)
            self._specs.add(out)
            return out

    def reload(self):
        logging.warn("Bundle '{0}' changed. Reloading".format(self.name))
        self._program = self.load_program(self.path,self._egs_version)

    def serialize(self) -> Any:
        logging.warn(str(self._specs.serialize()))
        if self._egs_version < 15:
            return [self.name,self._program,self._name_map]
        else:
            return {
                'bundle_name': self.name,
                'code': self._program,
                'specs': self._specs.serialize()
            }

class BundleSet:
    def __init__(self):
        self.bundles = []
        self._names = set()

    def add(self, bundle: Bundle):
        if bundle.name in self._names:
            return
        self.bundles.append(bundle)
        self._names.add(bundle.name)

    def merge(self, other: BundleSet):
        for bundle in other.bundles:
            self.add(bundle)

    def bundle_data(self) -> Any:
        return [ b.serialize() for b in self.bundles ]
