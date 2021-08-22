import logging
from typing import Optional
import toml
from collections import namedtuple
from core.config import SOURCES_TOML
from core.exceptions import RequestException
import requests
from ncd import NCDFileAccessor, NCDHttpAccessor

AccessItem = namedtuple('AccessItem',['variety','genome','chromosome'])

class AccessItem(object):
    def __init__(self, variety, genome = None, chromosome = None):
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
            elif self.variety == "jump":
                return "/".join(["jump.ncd"])
            elif self.variety == "seqs":
                return "/".join(["seqs",self.genome,self.chromosome])
            elif self.variety == "chrom-hashes":
                return "/".join(["common_files",self.genome,"chrom.hashes.ncd"])
            elif self.variety == "chrom-sizes":
                return "/".join(["common_files",self.genome,"chrom.sizes.ncd"])
            else:
                raise RequestException("unknown variety '{}'".format(self.variety))

    def stick(self) -> str:
        return ":".join([self.genome,self.chromosome]).replace('.','_')

class AccessMethod:
    def __init__(self):
        self.url = None
        self.file = None

class UrlAccessMethod(AccessMethod):
    def __init__(self, base_url: str, item: AccessItem):
        super().__init__()
        if not base_url.endswith("/"):
            base_url += "/"
        self.url = base_url + item.item_suffix()

    def get(self, offset: int, size: int):
        headers = { "Range": "bytes={0}-{1}".format(offset,offset+size) }
        r = requests.get(self.url, headers=headers)
        if r.status_code > 299:
            raise RequestException("bad data")
        return r.content

    def ncd(self):
        return NCDHttpAccessor(self.url)

class FileAccessMethod(AccessMethod):
    def __init__(self, base_path, item: AccessItem):
        super().__init__()
        if not base_path.endswith("/"):
            base_path += "/"
        self.file = base_path + item.item_suffix()

    def get(self, offset: int, size: int):
        with open(self.file,"r") as f:
            f.seek(0,offset)
            out = bytearray()
            while size > 0:
                more = f.read(size-len(out))
                if len(more) == 0:
                    raise RequestException("premature EOF")
                out += more
            return out

    def ncd(self):
        return NCDFileAccessor(self.file)

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

class DataSourceResolver:
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
