from __future__ import annotations
import logging, os.path
from model.programs import AllProgramSpecs, ProgramSpec
from typing import Any
from core.config import EGS_FILES

class Bundle:
    def __init__(self, begs_files, name: str, program_path: str, egs_version: int):
        self.begs_files = begs_files
        self.name = name
        self._specs = AllProgramSpecs()
        self._program = begs_files.load_program(program_path,egs_version)
        self._egs_version = egs_version

    def add_program(self, name: str, spec_path: str):
        if self._egs_version >= 15:
            if not os.path.exists(spec_path):
                raise Exception("missing spec file {}".format(spec_path))
            self._specs.add(ProgramSpec(name,spec_path))

    def monitor(self, begs_files, monitor):
        if self._monitor.check(self.name):
            logging.warn("Bundle '{0}' changed. Reloading".format(self.name))
            self._program = begs_files.load_program(monitor.path(self.name),self._egs_version)

    def serialize(self) -> Any:
        logging.warn(str(self._specs.serialize()))
        if self._egs_version < 15:
            return [self.name,self._program,self.begs_files.name_to_bundle_name[self.name]]
        else:
            return {
                'bundle_name': self.name,
                'code': self._program,
                'name_mapping': self.begs_files.name_to_bundle_name[self.name],
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
