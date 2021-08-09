class RequestException(BaseException):
    def __init__(self, message):
        self.message = message

