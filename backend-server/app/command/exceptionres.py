from .response import Response

class DataException(Exception):
    def to_response(self):
        return Response(1,self.args[0])
