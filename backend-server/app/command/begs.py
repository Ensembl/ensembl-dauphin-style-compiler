from __future__ import annotations

from command.bundle import Bundle
from core.config import BEGS_CONFIG, BEGS_FILES, OLD_BEGS_CONFIG, EGS_FILES
from typing import Any, List, Optional, Tuple;
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

class OldVersionedBegsFiles(object):
    def __init__(self, path: str, egs_version: int):
        with open(path) as f:
            toml_file = toml.loads(f.read())
        self.boot_program = toml_file["core"].get("boot",None)
        stick_authority = toml_file.get("stick-authority")
        if stick_authority != None:
            self.authority_startup_program = stick_authority.get("startup",None)
            self.authority_lookup_program = stick_authority.get("lookup",None)
            self.authority_jump_program = stick_authority.get("jump",None)
        else:
            self.authority_startup_program = None
            self.authority_lookup_program = None
            self.authority_jump_program = None
        self._bundles = {}
        self.name_to_bundle_name = {}
        self.name_to_bundle = {}
        self._monitor = BegsFilesMonitor()
        for (name_of_bundle,mapping) in toml_file["begs"].items():
            program_path = os.path.join(
                BEGS_FILES,
                "{}.begs".format(name_of_bundle)
            )
            self._monitor.add(name_of_bundle,program_path)
            self.name_to_bundle_name[name_of_bundle] = {}
            for (name_in_bundle,name_in_channel) in mapping.items():
                self.name_to_bundle[name_in_channel] = name_of_bundle
                self.name_to_bundle_name[name_of_bundle][name_in_channel] = name_in_bundle
            self._bundles[name_of_bundle] = Bundle(name_of_bundle,program_path,egs_version,name_map=self.name_to_bundle_name[name_of_bundle])

    def find_bundle(self, name: str) -> Optional[Bundle]:
        name = self.name_to_bundle.get(name,None)
        if name is None:
            return None
        return self._bundles.get(name,None)

    def boot_bundles(self) -> Any:
        return self._bundles.values()

    def load_program(self, program_path: str, egs_version: int) -> Any:
        with open(program_path,'rb') as f:
            return cbor2.loads(f.read())

    def add_bundle(self, bundle: Bundle) -> Any:
        bundle.monitor(self._monitor)
        return bundle.serialize(self.name_to_bundle_name[bundle.name])

# class VersionedBegsFiles(object):
#     def __init__(self, path: str, egs_version: int):
#         with open(path) as f:
#             toml_file = toml.loads(f.read())
#         self._specs_path = toml_file["specs"].get("path")
#         self._bundles = {}
#         self.boot_program = None # to go when v14 retired
#         self.name_to_bundle_name = {}
#         self.name_to_bundle = {}
#         self._monitor = BegsFilesMonitor()
#         for (name_of_bundle,mapping) in toml_file["begs"].items():
#             program_path = os.path.join(
#                 BEGS_FILES,
#                 "{}.begs".format(name_of_bundle)
#             )
#             self._bundles[name_of_bundle] = Bundle(name_of_bundle,program_path,egs_version)
#             self._monitor.add(name_of_bundle,program_path)
#             self.name_to_bundle_name[name_of_bundle] = {}
#             for (name_in_bundle,name_in_channel) in mapping.items():
#                 spec_file = os.path.join(
#                     BEGS_FILES, self._specs_path,
#                     "{}.toml".format(name_in_bundle)
#                 )
#                 self._bundles[name_of_bundle].add_program(name_in_bundle,spec_file)
#                 self.name_to_bundle[name_in_channel] = name_of_bundle
#                 self.name_to_bundle_name[name_of_bundle][name_in_channel] = name_in_bundle

#     def find_bundle(self, name: str) -> Optional[Bundle]:
#         name = self.name_to_bundle.get(name,None)
#         if name is None:
#             return None
#         return self._bundles.get(name,None)

#     def boot_bundles(self) -> Any:
#         return self._bundles.values()

#     def load_program(self, program_path: str, egs_version: int) -> Any:
#         with open(program_path,'rb') as f:
#             return f.read()

#     def add_bundle(self, bundle: Bundle) -> Any:
#         bundle.monitor(self._monitor)
#         return bundle.serialize()

class OneBegsFile:
    def __init__(self, name, toml_data, file_path):
        self._dir = os.path.dirname(file_path)
        self._load_config(name,toml_data)
        self._load_program()

    def _load_program(self):
        with open(self._path,'rb') as f:
            self._program_data = f.read()

    def _load_config(self, name, toml_data):
        self._name = name
        self._boot_for_version = []
        self._mapping = {}
        self._program_name = None
        self.load_toml(toml_data)
        for key in ("path","program_set","program_version","specs_path","programs"):
            if not hasattr(self,"_"+key):
                raise Exception("missing {} from program inventory for {}".format(key,name))
        self._program_version = int(self._program_version)
        self._boot_for_version = set(self._boot_for_version)
        self._path = os.path.join(self._dir,self._path)

    def name_for(self, begs_name) -> Tuple[str,str,int]:
        program_name = begs_name if self._program_name is None else self._program_name
        program_set = self._program_set
        program_version = self._program_version
        if begs_name in self._mapping:
            override = self._mapping[begs_name]
            if "program_set" in override:
                program_set = override["program_set"]
            if "program_name" in override:
                program_name = override["program_name"]
            if "program_version" in override:
                program_version = override["program_version"]
        return (program_set,program_name,program_version)

    def load_toml(self, data):
        if "general" in data:
            self.load_toml(data["general"])
        for key in ("path","program_set","program_version","specs_path","boot_for_version","programs"):
            if key in data:
                setattr(self,"_"+key,data[key])
        if "mapping" in data:
            self._mapping.update(data["mapping"])            

    def all_programs(self):
        programs = []
        for begs_name in self._programs:
            (program_set,program_name,program_version) = self.name_for(begs_name)
            programs.append((begs_name,program_set,program_name,program_version))
        return programs

    def path(self):
        return self._path

    def boot_versions(self):
        return self._boot_for_version

    def spec_files(self):
        files = {}
        for begs_name in self._programs:
            spec_file = os.path.join(
                BEGS_FILES, self._specs_path,
                "{}.toml".format(begs_name)
            )
            files[begs_name] = spec_file
        return files

class ProgramInventory:
    def __init__(self, egs_version):
        self._bundle = {}
        self._map_to_bundle = {}
        self._bundle_programs = {}
        self._boot_bundles = {}
        with open(BEGS_CONFIG) as f:
            toml_data = toml.loads(f.read())
            for (name,data) in toml_data.get("file",{}).items():
                one_file = OneBegsFile(name,data,BEGS_CONFIG)
                self._bundle[name] = Bundle(name,one_file.path(),egs_version)
                all_specs = one_file.spec_files()
                programs = []
                for (begs_name,program_set,program_name,program_version) in one_file.all_programs():
                    if begs_name not in all_specs:
                        raise Exception("missing spec for {}".format(begs_name))
                    self._bundle[name].add_program(begs_name,all_specs[begs_name])
                    full_name = (program_set,program_name,program_version)
                    self._map_to_bundle[full_name] = (name,begs_name)
                    programs.append(full_name)
                self._bundle_programs[name] = programs
                for version in one_file.boot_versions():
                    if version not in self._boot_bundles:
                        self._boot_bundles[version] = []
                    self._boot_bundles[version].append(name)

    def boot_bundles(self, egs_version):
        return [ self._bundle[name] for name in self._boot_bundles.get(egs_version,[]) ]

    def find_bundle(self, program_set: str, program_name: str, program_version: int):
        (bundle_name,_) = self._map_to_bundle[(program_set,program_name,program_version)]
        return self._bundle[bundle_name]

class BegsFiles(object):
    def __init__(self):
        self._versions = {}
        with open(OLD_BEGS_CONFIG) as f:
            toml_file = toml.loads(f.read())            
            logging.info("Loading begs files {0}".format(toml_file["version"]))
        for (version,filepath) in toml_file["version"].items():
            logging.info("Found begs files for version {0} in {1}".format(version,filepath))
            if int(version) < 15:
                self._versions[version] = OldVersionedBegsFiles(os.path.join(os.path.dirname(OLD_BEGS_CONFIG),filepath),int(version))
#            else:
#               self._versions[version] = VersionedBegsFiles(os.path.join(os.path.dirname(OLD_BEGS_CONFIG),filepath),int(version))

    def _bundle(self, version: Version) -> OldVersionedBegsFiles:
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

    def boot_bundles(self, version: Version) -> Any:
        return self._bundle(version).boot_bundles()

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
