"""Validate wire payloads before command routing or handler side effects."""

import math


def _array(value, length=None):
    return isinstance(value, (list, tuple)) and (length is None or len(value) == length)


def _uint(value):
    return type(value) is int and 0 <= value < 2**64


def _number(value):
    return (
        type(value) is float and math.isfinite(value)
        or type(value) is int and -(2**63) <= value < 2**64
    )


def _strings(value):
    return _array(value) and all(isinstance(item, str) for item in value)


def _index(value, table):
    return _uint(value) and value < len(table)


def is_valid_channel(value):
    # Older command-line clients send a channel string instead of a pair.
    return (
        isinstance(value, str) and len(value) >= 2
        or _strings(value) and len(value) == 2
    )


def _program(payload):
    return (
        _array(payload, 3)
        and isinstance(payload[0], str)
        and isinstance(payload[1], str)
        and _uint(payload[2])
        and payload[2] < 2**32
    )


def _one_string(payload):
    return _strings(payload) and len(payload) == 1


def _two_strings(payload):
    return _strings(payload) and len(payload) == 2


def _scope(scope):
    if not isinstance(scope, dict):
        return False
    for key, values in scope.items():
        if not isinstance(key, str) or not _strings(values):
            return False
        # These fields are indexed directly by data handlers; flags such as
        # "expanded" can legitimately contain an empty array.
        if key in ("datafile", "genome", "id", "type", "with_ids") and not values:
            return False
    return True


def _data(payload):
    if not _array(payload, 5):
        return False
    channel, name, region, scope, accept = payload
    return (
        is_valid_channel(channel)
        and isinstance(name, str)
        and _array(region, 3)
        and isinstance(region[0], str)
        and _uint(region[1])
        # Scale is a bit-shift count for the client's 64-bit coordinates.
        and region[1] < 64
        and _uint(region[2])
        and _scope(scope)
        and isinstance(accept, str)
    )


def _datastream(payload):
    if not isinstance(payload, dict):
        return False
    names, keys, points = (payload.get(key) for key in ("names", "keys", "datapoints"))
    return (
        _strings(names) and _strings(keys) and _array(points)
        and all(
            _array(point, 6)
            and _index(point[0], names)
            and _index(point[1], keys)
            and all(_uint(value) for value in point[2:])
            and point[3] in (0, 1)
            for point in points
        )
    )


def _programrun(payload):
    if not isinstance(payload, dict):
        return False
    names, points = payload.get("names"), payload.get("datapoints")
    return (
        _strings(names) and _array(points)
        and all(
            _array(point, 5)
            and _index(point[0], names)
            and _uint(point[1])
            and type(point[2]) is bool
            and all(_number(value) for value in point[3:])
            for point in points
        )
    )


def _general_point(point, tag_keys, tag_values, value_keys):
    return (
        _array(point, 4)
        and all(_array(part) for part in point)
        and len(point[0]) == len(point[1])
        and len(point[2]) == len(point[3])
        and all(_index(index, tag_keys) for index in point[0])
        and all(_index(index, tag_values) for index in point[1])
        and all(_index(index, value_keys) for index in point[2])
        and all(_number(value) for value in point[3])
    )


def _general(payload):
    if not _array(payload):
        return False
    for series in payload:
        if not _array(series, 5):
            return False
        name, tag_keys, tag_values, value_keys, points = series
        if not (
            isinstance(name, str)
            and all(_strings(table) for table in (tag_keys, tag_values, value_keys))
            and _array(points)
            and all(_general_point(point, tag_keys, tag_values, value_keys) for point in points)
        ):
            return False
    return True


def _metric(payload):
    if not isinstance(payload, dict) or not isinstance(payload.get("type"), str):
        return False
    # Other metric types are logged as supplied, without interpreting fields.
    return payload["type"] != "Client" or (
        _datastream(payload.get("datastream"))
        and _programrun(payload.get("programrun"))
        and _general(payload.get("general"))
    )


_VALIDATORS = {
    0: lambda payload: payload is None,
    1: _program,
    2: _one_string,
    4: _data,
    5: _one_string,
    6: _metric,
    7: _two_strings,
    8: _two_strings,
}


def is_valid_command_payload(command_type: int, payload) -> bool:
    validator = _VALIDATORS.get(command_type)
    # Unknown command IDs retain the protocol's unsupported-command response.
    return validator is None or validator(payload)
