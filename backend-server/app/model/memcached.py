from typing import Optional
from core.config import MEMCACHED
from command.response import Response
import logging
import hashlib
import cbor2

PYMEMCACHE_FOUND = True
try:
    from pymemcache.client.base import PooledClient
except:
    PYMEMCACHE_FOUND = False


class Memcached(object):
    def _check(self, host : str, port : str):
        self._client = PooledClient((host,port),max_pool_size=64)
        try:
            self._client.stats()
            self._available = True
        except:
            pass
        if self._available:
            logging.info("Using memcached at {0}:{1}",host,port)
        else:
            logging.warn("No memcached. THat's fine but will be slow.")

    def __init__(self):
        self._available = False
        if not PYMEMCACHE_FOUND:
            return
        (host,port) = MEMCACHED.split(':',1)
        self._check(host,port)

    def hashed_key(self,parts):
        value = hashlib.sha256()
        value.update(cbor2.dumps(parts))
        return value.hexdigest()

    def store_data(self, channel, name, panel, data):
        if not self._available:
            return
        key = self.hashed_key([channel,name,panel.dumps()])
        if len(data.payload) < 900_000:
            self._client.set(key,data.payload)

    def get_data(self, channel, name, panel) -> Optional[Response]:
        if not self._available:
            return None
        key = self.hashed_key([channel,name,panel.dumps()])
        value = self._client.get(key)
        if value == None:
            return None
        return Response(-1,value)
