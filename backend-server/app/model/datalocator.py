import logging
from pathlib import PurePosixPath
from typing import Any, Optional
import toml
from core.config import SOURCES_TOML
from core.exceptions import RequestException
import requests


def is_md5(checksum):
    if checksum and len(checksum) == 32:
        return True


class TrackFilepathError(ValueError):
    """Raised when the datafile path from Track API fails validation."""

def validate_track_filepath(filepath: str) -> str:
    """Validate a Track API filepath relative to a configured datasource root."""
    if not isinstance(filepath, str) or not filepath:
        raise TrackFilepathError("datafile path must be a non-empty string")
    if "\x00" in filepath:
        raise TrackFilepathError("datafile path must not contain null char")
    if "\\" in filepath:
        raise TrackFilepathError("datafile path must use POSIX separators")

    path = PurePosixPath(filepath)
    parts = filepath.split("/")
    if path.is_absolute() or any(part in ("", ".", "..") for part in parts):
        raise TrackFilepathError("datafile path must be a non-empty relative path without traversal")
    return filepath


class AccessItem(object):
    """ Class for accessing:
        - datasource pointer: (file or URL suffix) for a data type (variety).
        - properties: variety, (genome id), (chromosome)
        - stick string

    Args:
        variety (str): type of data requested
        genome (str): genome ID
        chromosome (str): chromosome name or hash (used for refget)
    """

    def __init__(self, variety: str, genome, chromosome: str = "", filepath: Optional[str] = None):
        self.variety: str = variety
        self.genome: str = genome
        self.chromosome: str = chromosome
        self.filepath: Optional[str] = filepath

    @classmethod
    def track_file(cls, filepath: str, genome, chromosome: str = ""):
        """Build an AccessItem for a datafile path from Track API."""
        return cls("track-file", genome, chromosome, validate_track_filepath(filepath))

    def stick(self) -> str:
        """Returns stick string (e.g. "a7335667-93e7-11ec-a39d-005056b38ce3:4")

        Returns:
            str: stick
        """
        return ":".join([self.genome, self.chromosome])


class AccessMethod:
    """ """

    def __init__(self):
        self.url = None
        self.file = None
        self.refget_url = None


class RefgetAccessMethod(AccessMethod):
    """

     Args:
         refget_url (str):
         item (AccessItem):
     """

    def __init__(self, refget_url: str, item: AccessItem):
        super().__init__()
        if not refget_url.endswith("/"):
            refget_url += "/"
        self.item = item
        self.url = refget_url + item.chromosome

    def get(self, offset: Optional[int] = None, size: Optional[int] = None):
        """

        Args:
            offset (:obj:'int', optional):
            size (:obj:'int', optional):

        Returns:
            Content of the response, in bytes.
        """

        headers = {}
        url_range = ""
        if offset is not None:
            headers["Range"] = "bytes={0}-{1}".format(offset, offset + size)
            url_range = f"?start={offset}&end={offset + size}"
        response = requests.get(self.url + url_range)
        if response.status_code > 299:
            raise RequestException(f"Refget error: {self.url + url_range} => {response.status_code}: {response.text}")
        return response.content


class MetadataApiClient:
    """Metadata API client with process-local and shared-cache layers."""

    _CHECKSUM_MISSING = {"missing": True}

    def __init__(self, metadata_url: str | None, cache=None):
        self._url = metadata_url.rstrip("/") + "/" if metadata_url else None
        self._cache = cache
        self._checksums: dict[tuple[str, str], str | None] = {}
        self._top_regions: dict[str, dict[str, int]] = {}

    def _url_for(self, path: str) -> str:
        if not self._url:
            raise RequestException("Metadata API URL is not configured")
        return self._url + path

    def _cache_get(self, key):
        return self._cache.get_metadata(key) if self._cache is not None else None

    def _cache_set(self, key, value):
        if self._cache is not None:
            self._cache.set_metadata(key, value)

    def get_checksum(self, genome: str, chromosome: str) -> str | None:
        key = (genome, chromosome)
        if key in self._checksums:
            return self._checksums[key]

        cache_key = ["checksum", genome, chromosome]
        cached = self._cache_get(cache_key)
        if cached == self._CHECKSUM_MISSING:
            self._checksums[key] = None
            return None
        if isinstance(cached, str) and cached:
            self._checksums[key] = cached
            return cached

        try:
            response = requests.get(
                self._url_for(f"genome/{genome}/checksum/{chromosome}"), timeout=5
            )
        except requests.RequestException as error:
            raise RequestException(
                f"Metadata checksum request failed for '{genome}:{chromosome}': {error}"
            ) from error
        if response.status_code == requests.codes.not_found:
            self._checksums[key] = None
            self._cache_set(cache_key, self._CHECKSUM_MISSING)
            return None
        if response.status_code > 299:
            raise RequestException(
                f"Metadata checksum request failed for '{genome}:{chromosome}': {response.status_code}"
            )
        checksum = response.text.strip()
        if not checksum:
            raise RequestException(f"Metadata checksum response is empty for '{genome}:{chromosome}'")
        self._checksums[key] = checksum
        self._cache_set(cache_key, checksum)
        return checksum

    def get_top_regions(self, genome: str) -> dict[str, int]:
        if genome in self._top_regions:
            return self._top_regions[genome]

        cache_key = ["top-regions", genome]
        cached = self._cache_get(cache_key)
        if isinstance(cached, dict) and all(
            isinstance(name, str) and bool(name) and isinstance(length, int)
            and not isinstance(length, bool) and length > 0
            for name, length in cached.items()
        ):
            self._top_regions[genome] = cached
            return cached

        try:
            response = requests.get(self._url_for(f"genome/{genome}/top-regions"), timeout=5)
        except requests.RequestException as error:
            raise RequestException(f"Metadata top-regions request failed for '{genome}': {error}") from error
        if response.status_code > 299:
            raise RequestException(
                f"Metadata top-regions request failed for '{genome}': {response.status_code}"
            )
        try:
            payload = response.json()
        except ValueError as error:
            raise RequestException(f"Metadata top-regions response is not JSON for '{genome}'") from error
        if not isinstance(payload, list):
            raise RequestException(f"Metadata top-regions response is not a list for '{genome}'")

        top_regions = {}
        for entry in payload:
            if not isinstance(entry, dict):
                raise RequestException(f"Metadata top-regions entry is invalid for '{genome}'")
            name = entry.get("name")
            length = entry.get("length")
            if not isinstance(name, str) or not name or not isinstance(length, int) or isinstance(length, bool) or length <= 0:
                raise RequestException(f"Metadata top-regions entry has invalid name or length for '{genome}'")
            if name in top_regions:
                raise RequestException(f"Metadata top-regions has duplicate chromosome '{name}' for '{genome}'")
            top_regions[name] = length

        self._top_regions[genome] = top_regions
        self._cache_set(cache_key, top_regions)
        return top_regions


class MetadataAccessMethod(AccessMethod):
    def __init__(self, client: MetadataApiClient, item: AccessItem):
        super().__init__()
        self._client = client
        self.item = item

    def get_checksum(self):
        return self._client.get_checksum(self.item.genome, self.item.chromosome)

    def get_top_regions(self):
        return self._client.get_top_regions(self.item.genome)


class UrlAccessMethod(AccessMethod):
    """

    Args:
        base_url (str):
        item (AccessItem):
    """

    def __init__(self, base_url: str, item: AccessItem):
        super().__init__()
        if item.filepath is None:
            raise RequestException(f"URL access requires a filepath, not variety '{item.variety}'")
        if not base_url.endswith("/"):
            base_url += "/"
        self.url = base_url + item.filepath

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

class FileAccessMethod(AccessMethod):
    """

    Args:
        base_path (str):
        item (:obj:'AccessItem'):
    """

    def __init__(self, base_path, item: AccessItem):
        super().__init__()
        if item.filepath is None:
            raise RequestException(f"File access requires a filepath, not variety '{item.variety}'")
        self.item = item
        if not base_path.endswith("/"):
            base_path += "/"
        self.base = base_path
        self.file = base_path + item.filepath

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
            raise RequestException(
                "Error accessing {0} (base={1}): {2}".format(self.file, self.base, e)
            )

class S3DataSource(object):
    """

    Args:
        data ():
    """

    def __init__(self, data, cache=None):
        self.url = data.get("url", None)
        self.refget_url = data.get("refget_url", None)
        self.metadata_url = data.get("metadata_url", None)
        self.metadata = MetadataApiClient(self.metadata_url, cache)
        if self.url is None:
            logging.critical("S3 driver config missing url")

    def resolve(self, item: AccessItem) -> Optional[AccessMethod]:
        if item.filepath is not None:
            method = UrlAccessMethod(self.url, item)
        elif is_md5(item.chromosome):
            method = RefgetAccessMethod(refget_url=self.refget_url, item=item)
        elif item.variety in ("chrom-hashes", "chrom-sizes"):
            method = MetadataAccessMethod(self.metadata, item=item)
        else:
            raise RequestException(f"Unsupported S3 datasource variety '{item.variety}'")
        return method


class FileDataSource(object):
    """
    Args:
        data (): datasources config from sources-<env>.toml
    """

    def __init__(self, data, cache=None):
        self.root = data.get("root", None)
        self.refget_url = data.get("refget_url", None)
        self.metadata_url = data.get("metadata_url", None)
        self.metadata = MetadataApiClient(self.metadata_url, cache)
        if self.root is None:
            logging.critical("File driver config missing root")

    def resolve(self, item: AccessItem) -> Optional[AccessMethod]:
        """

        Args:
            item ():

        Returns:

        """
        if item.filepath is not None:
            method = FileAccessMethod(self.root, item)
        elif is_md5(item.chromosome):
            method = RefgetAccessMethod(refget_url=self.refget_url, item=item)
        elif item.variety in ("chrom-hashes", "chrom-sizes"):
            method = MetadataAccessMethod(self.metadata, item=item)
        else:
            raise RequestException(f"Unsupported file datasource variety '{item.variety}'")
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
    """ """

    def __init__(self, version: int, cache=None):
        self._paths = {}
        self._redirect = {}
        self._blacklist = set()
        self._cache = cache
        self._load(SOURCES_TOML, version)

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
            self._paths[tuple(path)] = S3DataSource(data, self._cache)
        elif driver == "file":
            self._paths[tuple(path)] = FileDataSource(data, self._cache)
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
        sources_conf = config.get("source", {})
        for source in sources_conf:
            source_conf = sources_conf[source]
            min_version = source_conf.get("min_version", None)
            if min_version is not None and version < min_version:
                continue
            max_version = source_conf.get("max_version", None)
            if max_version is not None and version > max_version:
                continue
            logging.info("Choosing source '{}' for version {}".format(source, version))
            return source_conf
        raise RequestException("no source for version {}".format(version))

    def _load(self, source, version: int):
        """

        Args:
            source ():

        Returns:

        """
        toml_data = toml.load(source)
        self._add([], self._select_source(toml_data, version))
        self._add_redirect([], toml_data.get("redirect", {}))

    def get(self, item: AccessItem) -> Optional[AccessMethod]:
        """

        Args:
            item (AccessItem):

        Returns:
            (S3/File)Datasource.resolve -> (File/URL/Refget)AccessMethod 
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
