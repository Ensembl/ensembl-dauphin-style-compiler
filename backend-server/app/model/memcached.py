from datetime import datetime
from types import prepare_class
from typing import Optional, Tuple
from core.config import MEMCACHED
from command.response import Response
import logging
import hashlib
import cbor2
import time

"""
Attributes:
    PYMEMCACHE_FOUND (boolean)
    STARTUP_PERIOD (int)
    STARTUP_INTERVAL (int)
    REGULAR_INVERVAL (int)
"""
PYMEMCACHE_FOUND = True
try:
    from pymemcache.client.base import PooledClient
except:
    PYMEMCACHE_FOUND = False

STARTUP_PERIOD = 300
STARTUP_INTERVAL = 1
REGULAR_INTERVAL = 300


class Memcached(object):
    def __init__(self, prefix):
        """

        Args:
            prefix ():
        """
        self._start_time = time.time()
        self._last_check = 0
        self._last_bump_check = None
        self._bump = None
        self._prefix = prefix
        self._available = False
        if not PYMEMCACHE_FOUND:
            logging.warn("missing pymemcached. Cannot use memcache")
            return
        (host, port) = MEMCACHED.split(':', 1)
        logging.warn("trying memcached {0}:{1}".format(host, port))
        self._client = PooledClient((host, port), max_pool_size=64)
        self._check()

    def _check(self):
        """

        Returns:

        """
        if self._available:
            return True
        try:
            self._client.stats()
            self._available = True
        except:
            pass
        if self._available:
            logging.warn("Memcached has started. Will use.")
        else:
            logging.warn("No memcached. That's fine but will be slow.")
        return self._available

    def _is_available(self):
        """

        Returns:

        """
        if self._available:
            return True
        now = time.time()
        interval = STARTUP_INTERVAL if now - self._start_time < STARTUP_PERIOD else REGULAR_INTERVAL
        if self._last_check + interval < now:
            self._check()
            self._last_check = now
        return self._available

    def _get_bump(self):
        """

        Returns:
            None
        """
        now = time.time()
        if self._last_bump_check is None or now - self._last_bump_check > 30:
            self._last_bump_check = now
            new_value = self._client.get(self._prefix + ":" + "bump")
            if new_value is not None:
                self._bump = new_value

    def hashed_key(self, parts):
        """

        Args:
            parts ():

        Returns:
            string
        """
        value = hashlib.sha256()
        self._get_bump()
        value.update(cbor2.dumps([self._prefix, self._bump, parts]))
        return value.hexdigest()

    def store_data(self, channel, name, panel, data):
        """

        Args:
            channel ():
            name ():
            panel ():
            data ():

        Returns:
            None
        """
        if not self._is_available():
            return
        key = self.hashed_key([channel, name, panel.dumps()])
        if len(data.payload) < 900_000:
            self._client.set(key, data.payload)

    def get_data(self, channel, name, panel) -> Optional[Response]:
        """

        Args:
            channel ():
            name ():
            panel ():

        Returns:
            Optional[Response]: if validated and value isn't None else None.
        """
        if not self._is_available():
            return None
        key = self.hashed_key([channel, name, panel.dumps()])
        value = self._client.get(key)
        if value is None:
            return None
        return Response(-1, value)

    def get_jump(self, name: str) -> Optional[Tuple[str, int, int]]:
        """

        Args:
            name (str):

        Returns:
            Optional[Tuple[str, int, int]]: if validated and value isn't None else None.
        """
        if not self._is_available():
            return None
        key = self.hashed_key(["jump", name])
        value = self._client.get(key)
        return cbor2.loads(value) if value is not None else None

    def set_jump(self, name: str, stick: str, start: int, end: int):
        """

        Args:
            name (str):
            stick (str):
            start (int):
            end (int):

        Returns:
            None
        """
        if not self._is_available():
            return
        key = self.hashed_key(["jump", name])
        self._client.set(key, cbor2.dumps([stick, start, end]))
