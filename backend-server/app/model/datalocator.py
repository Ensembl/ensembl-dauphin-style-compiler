import logging
from typing import Any, Optional
import toml
from collections import namedtuple
from core.config import SOURCES_TOML
from core.exceptions import RequestException
import requests
from ncd import NCDFileAccessor, NCDHttpAccessor
"""
Attributes:
    AccessItem (namedtuple):
"""
AccessItem = namedtuple('AccessItem', ['variety', 'genome', 'chromosome'])


class AccessItem(object):
    """
    Args:
            variety (str):
            genome (str):
            chromosome (str):
    """

    variety_map = {
        "contigs" : "contigs/{genome}/contigs.bb",
        "transcripts" : "genes_and_transcripts/{genome}/transcripts.bb",
        "gc" : "gc/{genome}/gc.bw",
        "variant-labels" : "variants/{genome}/variant-labels.bb",
        "jump" : "jump/{genome}/jump.ncd",
        "seqs" : "seqs/{genome}/{chromosome}",
        "chrom-hashes": "common_files/{genome}/chrom.hashes.ncd",
        "chrom-sizes" : "common_files/{genome}/chrom.sizes.ncd",
        "species-list": "species.txt",
        "variant-summary" : "variants/{genome}/variant-summary.bw",
        "v_2-summary" : "variants/{genome}/variant-summary-2.bw",
        "variant-summary-3" : "variants/{genome}/variant-summary.bw",
        "variant-summary-4" : "variants/{genome}/variant-summary.bw",
        "variant-summary-5" : "variants/{genome}/variant-summary.bw",
        "variant-summary-6" : "variants/{genome}/variant-summary.bw",
    }

    def __init__(self, variety: str, genome: str = None, chromosome: str = None):
        self.variety: str = variety
        self.genome: str = genome
        self.chromosome: str = chromosome

    def item_suffix(self) -> str:
        """

        Returns:
            variety string.

        """
        if self.variety in AccessItem.variety_map:
            return AccessItem.variety_map[self.variety].format(genome = self.genome, chromosome = self.chromosome)
        else:
            raise RequestException("unknown variety '{}'".format(self.variety))

    def stick(self) -> str:
        """

        Returns:
            str:
        """
        return ":".join([self.genome, self.chromosome]).replace('.', '_')


class AccessMethod:
    """

    """

    def __init__(self):
        self.url = None
        self.file = None


class UrlAccessMethod(AccessMethod):
    """

    Args:
        base_url (str):
        item (AccessItem):
    """

    def __init__(self, base_url: str, item: AccessItem):
        super().__init__()
        if not base_url.endswith("/"):
            base_url += "/"
        self.url = base_url + item.item_suffix()

    def get(self, offset: Optional[int] = None, size: Optional[int] = None):
        """

        Args:
            offset (:obj:'int', optional):
            size (:obj:'int', optional):

        Returns:
            Content of the response, in bytes.
        """
        headers = {}
        if offset is not None:
            headers["Range"] = "bytes={0}-{1}".format(offset, offset + size)
        response = requests.get(self.url, headers=headers)
        if response.status_code > 299:
            raise RequestException("bad data")
        return response.content

    def ncd(self):
        """
        Returns:
             bytearray:
        """
        return NCDHttpAccessor(self.url)


class FileAccessMethod(AccessMethod):
    """

    Args:
        base_path (str):
        item (:obj:'AccessItem'):
    """

    def __init__(self, base_path, item: AccessItem):
        super().__init__()
        if not base_path.endswith("/"):
            base_path += "/"
        self.base = base_path
        self.file = base_path + item.item_suffix()

    def get(self, offset: Optional[int] = None, size: Optional[int] = None):
        """

        Args:
            offset ():
            size ():

        Returns:
            bytearray
        """
        out = bytearray()
        try:
            with open(self.file, "rb") as f:
                if offset is not None:
                    f.seek(offset, 0)
                    while size - len(out) > 0:
                        more = f.read(size - len(out))
                        if len(more) == 0:
                            raise RequestException("premature EOF")
                        out += more
                else:
                    while True:
                        more = f.read(4096)
                        if len(more) == 0:
                            return out
                        out += more
                return out
        except Exception as e:
            raise RequestException("Error accessing {0} (base={1}): {2}".format(self.file, self.base, e))

    def ncd(self):
        """

        Returns:

        """
        return NCDFileAccessor(self.file)


class S3DataSource(object):

    """

    Args:
        data ():
    """
    def __init__(self, data):

        self.url = data.get("url", None)
        if self.url is None:
            logging.critical("S3 driver config missing url")

    def resolve(self, item: AccessItem) -> Optional[AccessMethod]:
        method = UrlAccessMethod(self.url, item)
        return method


class FileDataSource(object):
    """
    Args:
        data ():
    """

    def __init__(self, data):
        self.root = data.get("root", None)
        if self.root is None:
            logging.critical("File driver config missing root")

    def resolve(self, item: AccessItem) -> Optional[AccessMethod]:
        """

        Args:
            item ():

        Returns:

        """
        method = FileAccessMethod(self.root, item)
        return method


class NoneDataSource(object):

    @staticmethod
    def resolve(_item: AccessItem) -> Optional[AccessMethod]:
        """

        Args:
            _item (AccessItem):

        Returns:
            None
        """
        return None


class DataSourceResolver:
    """

    """

    def __init__(self, version: int):
        self._paths = {}
        self._redirect = {}
        self._blacklist = set()
        self._load(SOURCES_TOML,version)

    def _add_here(self, path, data):
        """

        Args:
            path ():
            data (dict):

        Returns:
            None

        """
        driver = data["driver"]
        if driver == "s3":
            self._paths[tuple(path)] = S3DataSource(data)
        elif driver == "file":
            self._paths[tuple(path)] = FileDataSource(data)
        elif driver == "none":
            self._paths[tuple(path)] = NoneDataSource()
        else:
            logging.critical("No such driver '{}'".format(driver))

    def _add(self, path, data):
        """

        Args:
            path ():
            data ():

        Returns:
            None
        """
        if "driver" in data and data["driver"] and not (type(data["driver"]) is dict):
            self._add_here(path, data)
        for (more_path, new_data) in data.items():
            if type(new_data) is dict:
                self._add(path + [more_path], new_data)

    def _add_redirect(self, path, data):
        """

        Args:
            path ():
            data ():

        Returns:
            None
        """
        if "upstream" in data and not (type(data["upstream"]) is dict):
            self._redirect[tuple(path)] = data["upstream"]
        for (more_path, new_data) in data.items():
            if type(new_data) is dict:
                self._add_redirect(path + [more_path], new_data)

    def _select_source(self, config, version: int) -> Any:
        sources_conf = config.get('source',{})
        for source in sources_conf:
            source_conf = sources_conf[source]
            min_version = source_conf.get('min_version',None)
            if min_version is not None and version < min_version:
                continue
            max_version = source_conf.get('max_version',None)
            if max_version is not None and version > max_version:
                continue
            logging.info("Choosing source '{}' for version {}".format(source,version))
            return source_conf
        raise RequestException("no source for version {}".format(version))

    def _load(self, source, version: int):
        """

        Args:
            source ():

        Returns:

        """
        toml_data = toml.load(source)
        self._add([], self._select_source(toml_data,version))
        self._add_redirect([], toml_data.get('redirect', {}))

    def get(self, item: AccessItem) -> Optional[AccessMethod]:
        """

        Args:
            item (AccessItem):

        Returns:

        """
        pattern = tuple([item.variety, item.genome, item.chromosome])
        if pattern in self._paths:
            return self._paths[pattern].resolve(item)

        pattern = tuple([item.variety, item.genome])
        if pattern in self._paths:
            return self._paths[pattern].resolve(item)

        pattern = tuple([item.variety])
        if pattern in self._paths:
            return self._paths[pattern].resolve(item)

        pattern = tuple()
        if pattern in self._paths:
            return self._paths[pattern].resolve(item)
        return None

    def find_override(self, prefix):
        """

        Args:
            prefix ():

        Returns:
            if v exists return v else return None.
        """
        for end in reversed(range(0, len(prefix) + 1)):
            v = self._redirect.get(tuple(prefix[0:end]), None)
            if v is not None:
                return v if v else None
        return None
