import itertools

class Getter:
    def __init__(self, our_config, normal_sources, aux_sources, callback):
        self._our_config = our_config
        self._normal_sources = normal_sources
        self._aux_sources = aux_sources
        self._callback = callback

    def _get_value(self, row, key):
        for source in self._our_config[key]:
            value = source.get(row)
            if value:
                return value
        return None

    def get(self, row, state):
        values = []
        is_list = None
        for (source_key,type_) in self._normal_sources:
            value = self._get_value(row,source_key)
            if is_list is None:
                is_list = (not isinstance(value,type_)) and isinstance(value,list)
            if is_list:
                value = [type_(x) for x in value]
            else:
                value = type_(value)
            values.append(value)
        for (source_key,type_,default) in self._aux_sources:
            if source_key in self._our_config and self._our_config[source_key] is not None:
                value = self._get_value(row,source_key)
            else:
                value = default
            if is_list:
                this_is_list = (not isinstance(value,type_)) and isinstance(value,list)
                if not this_is_list:
                    value = itertools.repeat(value)
            values.append(value)
        if is_list:
            for row in zip(*values):
                self._callback(state,*row)
        else:
            self._callback(state,*values)
        return values
