from typing import Any

class Version(object):
    def __init__(self, payload: Any):
        self._payload = payload

    def get(self, key: str, default: Any) -> Any:
        if self._payload == None:
            return default
        return self._payload.get(key,default)

    def get_egs(self) -> int:
       return  self.get("egs",0)

    def encode(self) -> Any:
        return {
            "egs": self.get_egs()
        }