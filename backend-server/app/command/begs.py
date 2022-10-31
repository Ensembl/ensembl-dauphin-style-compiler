from __future__ import annotations
from core.config import BEGS_FILES, BEGS_CONFIG
from typing import Any, List, Optional;
import logging
import time
import cbor2
import toml
import os.path
from os import stat
from model.version import Version

class UnknownVersionException(Exception):
    pass

class BegsFilesMonitor(object):
    def __init__(self):
        self._paths = {}
        self._mtimes = {}
        pass

    def add(self, name: str, path: str):
        self._paths[name] = path
        self.check(name)

    def check(self, name: str) -> bool:
        path = self._paths[name]
        if path == None:
            return False
        old_mtime = self._mtimes.get(path,0)
        new_mtime = os.stat(path).st_mtime
        if old_mtime != new_mtime:
            self._mtimes[path] = new_mtime
            return True
        else:
            return False

    def path(self, name: str) -> Optional[str]:
        return self._paths.get(name)

class VersionedBegsFiles(object):
    def __init__(self, path: str, egs_version: int):
        with open(path) as f:
            toml_file = toml.loads(f.read())            
        self.boot_program = toml_file["core"].get("boot",None) # gone v15 onwards 
        stick_authority = toml_file.get("stick-authority")
        if stick_authority != None:
            self.authority_startup_program = stick_authority.get("startup",None) # gone v15 onwards 
            self.authority_lookup_program = stick_authority.get("lookup",None) # gone v15 onwards
            self.authority_jump_program = stick_authority.get("jump",None) # gone v15 onwards
        else:
            self.authority_startup_program = None
            self.authority_lookup_program = None
            self.authority_jump_program = None
        self.name_to_bundle_name = {}
        self.program_map = {}
        self._monitor = BegsFilesMonitor()
        for (name_of_bundle,mapping) in toml_file["begs"].items():
            program_path = os.path.join(
                BEGS_FILES,
                "{}.begs".format(name_of_bundle)
            )
            self._monitor.add(name_of_bundle,program_path)
            self.name_to_bundle_name[name_of_bundle] = {}
            for (name_in_bundle,name_in_channel) in mapping.items():
                self.program_map[name_in_channel] = Bundle(self,name_of_bundle,program_path,egs_version)
                self.name_to_bundle_name[name_of_bundle][name_in_channel] = name_in_bundle

    def find_bundle(self, name: str) -> Optional[Bundle]:
        return self.program_map.get(name,None)

    def all_bundles(self) -> Any:
        return self.program_map.values()

    def load_program(self, program_path: str, egs_version: int) -> Any:
        with open(program_path,'rb') as f:
            if egs_version < 15:
                return cbor2.loads(f.read())
            else:
                return f.read()

    def add_bundle(self, bundle: Bundle) -> Any:
        bundle.monitor(self,self._monitor)
        return bundle.serialize()

class BegsFiles(object):
    def __init__(self):
        self._versions = {}
        with open(BEGS_CONFIG) as f:
            toml_file = toml.loads(f.read())            
            logging.info("Loading begs files {0}".format(toml_file["version"]))
        for (version,filepath) in toml_file["version"].items():
            logging.info("Found begs files for version {0} in {1}".format(version,filepath))
            self._versions[version] = VersionedBegsFiles(os.path.join(os.path.dirname(BEGS_CONFIG),filepath),int(version))

    def _bundle(self, version: Version) -> VersionedBegsFiles:
        egs_version = version.get_egs()
        bundle = self._versions.get(str(egs_version),None)
        if bundle == None:
            raise UnknownVersionException("Unknown egs version {0}".format(egs_version))
        return bundle

    def versions(self) -> List[int]:
        return [int(x) for x in self._versions.keys()]

    def boot_program(self, version: Version) -> str:
        return self._bundle(version).boot_program

    def find_bundle(self, name: str, version: Version) -> Optional[Bundle]:
        return self._bundle(version).find_bundle(name)

    def all_bundles(self, version: Version) -> Any:
        return self._bundle(version).all_bundles()

    def add_bundle(self, bundle_name: str, version: Version) -> Any:
        return self._bundle(version).add_bundle(bundle_name)

    # There is no authority_lookup_program from v15 on
    def authority_startup_program(self, version: Version):
        return self._bundle(version).authority_startup_program

    # There is no authority_lookup_program from v15 on
    def authority_lookup_program(self, version: Version):
        return self._bundle(version).authority_lookup_program

    # There is no authority_jump_program from v15 on
    def authority_jump_program(self, version: Version):
        return self._bundle(version).authority_jump_program

class Bundle(object):
    def __init__(self, begs_files: BegsFiles, name: str, program_path: str, egs_version: int):
        self.begs_files = begs_files
        self.name = name
        self._program = begs_files.load_program(program_path,egs_version)
        self._egs_version = egs_version

    def monitor(self, begs_files: BegsFiles, monitor: BegsFilesMonitor):
        if self._monitor.check(self.name):
            logging.warn("Bundle '{0}' changed. Reloading".format(self.name))
            self._program = begs_files.load_program(monitor.path(self.name),self._egs_version)

    def serialize(self) -> Any:
        return [self.name,self._program,self.begs_files.name_to_bundle_name[self.name]]

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
