class RequestException(BaseException):
    def __init__(self, message):
        self.message = message


class UpstreamRequestError(Exception):
    """An expected upstream HTTP request failure, with added context for debugging."""

    def __init__(self, service, method, url, detail, *, upstream_status=None, status_code=502):
        self.service = service
        self.method = method
        self.url = url
        self.detail = str(detail)[:2048]
        self.upstream_status = upstream_status
        self.status_code = status_code
        super().__init__(f"{service} {method} {url}: {upstream_status or 'no response'}: {self.detail}")

