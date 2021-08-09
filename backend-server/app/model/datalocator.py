import logging
from typing import Optional
import toml
from collections import namedtuple
from core.config import SOURCES_TOML
from core.exceptions import RequestException

AccessItem = namedtuple('AccessItem',['variety','genome','chromosome'])

class AccessItem(object):
    def __init__(self, variety, genome, chromosome):
        self.variety = variety
        self.genome = genome
        self.chromosome = chromosome

    def item_suffix(self) -> str:
            if self.variety == "contigs":
                return "/".join(["contigs",self.genome,"contigs.bb"])
            elif self.variety == "transcripts":
                return "/".join(["genes_and_transcripts",self.genome,"transcripts.bb"])
            elif self.variety == "gc":
                return "/".join(["gc",self.genome,"gc.bw"])
            elif self.variety == "variant-summary":
                return "/".join(["variants",self.genome,"variant-summary.bw"])
            else:
                raise RequestException("unknown variety '{}'".format(self.variety))

    def stick(self) -> str:
        return ":".join([self.genome,self.chromosome]).replace('.','_')

class AccessMethod(object):
    def __init__(self):
        self.url = None
        self.file = None

class UrlAccessMethod(AccessMethod):
    def __init__(self, base_url: str, item: AccessItem):
        super().__init__()
        if not base_url.endswith("/"):
            base_url += "/"
        self.url = base_url + item.item_suffix()

class FileAccessMethod(AccessMethod):
    def __init__(self, base_path, item: AccessItem):
        super().__init__()
        if not base_path.endswith("/"):
            base_path += "/"
        self.file = base_path + item.item_suffix()

class S3DataSource(object):
    def __init__(self,data):
        self.url = data.get("url",None)
        if self.url == None:
            logging.critical("S3 driver config missing url")

    def resolve(self, item: AccessItem) -> Optional[AccessMethod]:
        method = UrlAccessMethod(self.url,item)
        return method

class FileDataSource(object):
    def __init__(self,data):
        self.root = data.get("root",None)
        if self.root == None:
            logging.critical("File driver config missing root")

    def resolve(self, item: AccessItem) -> Optional[AccessMethod]:
        method = FileAccessMethod(self.root,item)
        return method

class DataSourceResolver(object):
    def __init__(self):
        self._paths = {}
        self._load(SOURCES_TOML)

    def _add_here(self,path,data):
        driver = data["driver"]
        if driver == "s3":
            self._paths[tuple(path)] = S3DataSource(data)
        elif driver == "file":
            self._paths[tuple(path)] = FileDataSource(data)
        else:
            logging.critical("No such driver '{}'".format(driver))

    def _add(self,path,data):
        if "driver" in data and data["driver"] and not (type(data["driver"]) is dict):
            self._add_here(path,data)
        for (more_path,new_data) in data.items():
            if type(new_data) is dict:
                self._add(path+[more_path],new_data)

    def _load(self,source):
        toml_data = toml.load(source)
        self._add([],toml_data.get('source',{}))

    def get(self, item: AccessItem) -> Optional[AccessMethod]:
        pattern = tuple([item.variety,item.genome,item.chromosome])
        if pattern in self._paths:
            return self._paths[pattern].resolve(item)

        pattern = tuple([item.variety,item.genome])
        if pattern in self._paths:
            return self._paths[pattern].resolve(item)

        pattern = tuple([item.variety])
        if pattern in self._paths:
            return self._paths[pattern].resolve(item)

        pattern = tuple()
        if pattern in self._paths:
            return self._paths[pattern].resolve(item)
        return None
