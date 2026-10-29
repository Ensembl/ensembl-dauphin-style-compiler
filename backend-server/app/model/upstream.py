"""Normalize HTTP dependency failures without catching application bugs."""

import requests

from core.exceptions import UpstreamRequestError


def request_upstream(service, method, url, *, allowed_statuses=(), invalid_request_statuses=(), **kwargs):
    kwargs.setdefault("timeout", 5)
    try:
        response = getattr(requests, method.lower())(url, **kwargs)
    except requests.Timeout as error:
        raise UpstreamRequestError(service, method, url, error, status_code=504) from error
    except requests.RequestException as error:
        raise UpstreamRequestError(service, method, url, error) from error
    if not 200 <= response.status_code < 300 and response.status_code not in allowed_statuses:
        response_url = getattr(response, "url", None)
        raise UpstreamRequestError(
            service, method, response_url if isinstance(response_url, str) else url,
            response.text, upstream_status=response.status_code,
            # Only explicitly selected input rejections become HTTP 400
            status_code=400 if response.status_code in invalid_request_statuses else 502,
        )
    return response


def request_json(service, method, url, **kwargs):
    response = request_upstream(service, method, url, **kwargs)
    try:
        return response.json()
    except ValueError as error:
        raise UpstreamRequestError(
            service, method, url, "Invalid JSON response", upstream_status=response.status_code,
        ) from error
