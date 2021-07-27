from core.config import BEGS_FILES, BEGS_CONFIG
from typing import Any;
import logging
import toml
import time
import cbor2
import os.path
from os import stat

class BegsFilesMonitor(object):
    def __init__(self):
        pass

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
        for (name_of_bundle,mapping) in toml_file["begs"].items():
            program_path = os.path.join(
                BEGS_FILES,
                "{}.begs".format(name_of_bundle)
            )
            self.program[name_of_bundle] = self.load_program(program_path)
            self.bundle_contents[name_of_bundle] = {}
            for (name_in_bundle,name_in_channel) in mapping.items():
                self.program_map[name_in_channel] = (name_of_bundle,name_in_bundle)
                self.bundle_contents[name_of_bundle][name_in_channel] = name_in_bundle
        self._monitor = BegsFilesMonitor()

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
        return Bundle(self,bundle_name).serialize(self.program[bundle_name])

class Bundle(object):
    def __init__(self, begs_files: BegsFiles, name: str):
        self.begs_files = begs_files
        self.name = name

    def serialize(self, program: Any) -> Any:
        return [self.name,program,self.begs_files.bundle_contents[self.name]]
