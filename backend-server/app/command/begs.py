from core.config import BEGS_FILES, BEGS_CONFIG
from typing import Any, List, Optional;
import logging
import toml
import time
import cbor2
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
        self.boot_program = toml_file["core"]["boot"]
        stick_authority = toml_file.get("stick-authority")
        if stick_authority != None:
            self.authority_startup_program = stick_authority["startup"]
            self.authority_lookup_program = stick_authority.get("lookup",None) # gone v15 onwards
            self.authority_jump_program = stick_authority.get("jump",None) # gone v15 onwards
        else:
            self.authority_startup_program = None
            self.authority_lookup_program = None
            self.authority_jump_program = None
        self.bundle_contents = {}
        self.program_map = {}
        self.program = {}
        self._monitor = BegsFilesMonitor()
        for (name_of_bundle,mapping) in toml_file["begs"].items():
            program_path = os.path.join(
                BEGS_FILES,
                "{}.begs".format(name_of_bundle)
            )
            self._monitor.add(name_of_bundle,program_path)
            self.program[name_of_bundle] = self.load_program(program_path,egs_version)
            self.bundle_contents[name_of_bundle] = {}
            for (name_in_bundle,name_in_channel) in mapping.items():
                self.program_map[name_in_channel] = (name_of_bundle,name_in_bundle)
                self.bundle_contents[name_of_bundle][name_in_channel] = name_in_bundle

    def find_bundle(self, name: str) -> str:
        v = self.program_map[name]
        if v != None:
            return v[0]
        return None

    def all_bundles(self) -> Any:
        return self.program.keys()

    def load_program(self, program_path: str, egs_version: int) -> Any:
        with open(program_path,'rb') as f:
            if egs_version < 15:
                return cbor2.loads(f.read())
            else:
                return f.read()

    def add_bundle(self, bundle_name: str, version: Version) -> Any:
        if self._monitor.check(bundle_name):
            logging.warn("Bundle '{0}' changed. Reloading".format(bundle_name))
            egs_version = version.get_egs()
            self.program[bundle_name] = self.load_program(self._monitor.path(bundle_name),egs_version)
        return Bundle(self,bundle_name).serialize(self.program[bundle_name])

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

    def find_bundle(self, name: str, version: Version) -> str:
        return self._bundle(version).find_bundle(name)

    def all_bundles(self, version: Version) -> Any:
        return self._bundle(version).all_bundles()

    def add_bundle(self, bundle_name: str, version: Version) -> Any:
        return self._bundle(version).add_bundle(bundle_name,version)

    def authority_startup_program(self, version: Version):
        return self._bundle(version).authority_startup_program

    # There is no authority_lookup_program from v15 on
    def authority_lookup_program(self, version: Version):
        return self._bundle(version).authority_lookup_program

    # There is no authority_jump_program from v15 on
    def authority_jump_program(self, version: Version):
        return self._bundle(version).authority_jump_program

class Bundle(object):
    def __init__(self, begs_files: BegsFiles, name: str):
        self.begs_files = begs_files
        self.name = name

    def serialize(self, program: Any) -> Any:
        return [self.name,program,self.begs_files.bundle_contents[self.name]]
