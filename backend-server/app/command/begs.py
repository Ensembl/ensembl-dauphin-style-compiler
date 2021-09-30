from core.config import BEGS_FILES, BEGS_CONFIG
from typing import Any, Optional;
import logging
import toml
import time
import cbor2
import os.path
from os import stat

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

class BegsFiles(object):
    def __init__(self):
        with open(BEGS_CONFIG) as f:
            toml_file = toml.loads(f.read())            
        self.boot_program = toml_file["core"]["boot"]
        stick_authority = toml_file.get("stick-authority")
        if stick_authority != None:
            self.stickauthority_startup_program = stick_authority["startup"]
            self.stickauthority_lookup_program = stick_authority["lookup"]
            self.stickauthority_jump_program = stick_authority["jump"]
        else:
            self.stickauthority_startup_program = None
            self.stickauthority_lookup_program = None
            self.stickauthority_jump_program = None
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
            self.program[name_of_bundle] = self.load_program(program_path)
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

    def load_program(self, program_path: str) -> Any:
        with open(program_path,'rb') as f:
            return cbor2.loads(f.read())

    def add_bundle(self, bundle_name: str) -> Any:
        if self._monitor.check(bundle_name):
            logging.warn("Bundle '{0}' changed. Reloading".format(bundle_name))
            self.program[bundle_name] = self.load_program(self._monitor.path(bundle_name))
        return Bundle(self,bundle_name).serialize(self.program[bundle_name])

class Bundle(object):
    def __init__(self, begs_files: BegsFiles, name: str):
        self.begs_files = begs_files
        self.name = name

    def serialize(self, program: Any) -> Any:
        return [self.name,program,self.begs_files.bundle_contents[self.name]]
