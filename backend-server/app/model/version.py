from typing import Any


class Version(object):
    """

    Args:
        payload (Any):
    """
    def __init__(self, payload: Any):
        self._payload = payload

    def get(self, key: str, default: Any) -> Any:
        """

        Args:
            key (str):
            default (any):

        Returns:
            Any:
        """
        if self._payload is None:
            return default
        return self._payload.get(key, default)

    def get_egs(self) -> int:
        """

        Returns:
            int:
        """
        return self.get("egs", 0)

    def encode(self) -> Any:
        """

        Returns:
            Any:
        """
        return {
            "egs": self.get_egs()
        }
