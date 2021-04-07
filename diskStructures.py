SHORT_MODE_LIMIT = 100
BLOCK_SIZE = 512 # in bytes
LBA_SIZE = 510

LBA_NUMBER = 32

class Address:
    def __init__(self, lba=-1, index=-1):
        self.lba = lba
        self.index = index

class Header:
    def __init__(self, name, type_):
        self.name = name
        self.type = type_
        self.address = Address()
    
    def get_address(self):
        # TODO
        pass
class UstarFile:
    def __init__(self):
        pass

class File(UstarFile):
    def __init__(self, data, name):
        self.header = Header(name, 2)
        self.data = data
    
    def __len__(self):
        return len(self.data) # in bytes
    
    def get_data(self):
        return self.data[:]
    
    def mode(self):
        if len(self) < SHORT_MODE_LIMIT * BLOCK_SIZE:
            return 0
        else:
            return 1

class Dir(UstarFile):
    def __init__(self, data, name):
        self.header = Header(name, 1)
        self.data = data
        self.files = []

    def add_file(self, file : UstarFile):
        self.files.append(file)
    
    def get_data(self):
        data = []
        # get data segment of the dir
        for e in self.data:
            header = e.header
            name = header.name
            len_name = len(name)
            address = header.get_address()
            for i in range(28):
                if i < len_name:
                    data.append(ord(name[i]))
                else:
                    data.append(0)
            data.append(address[0])
            data.append(address[1])
        return data
    
    # Length without header
    def __len__(self):
        return len(self.files) * 32 # in bytes
    
    def mode(self):
        if len(self) < SHORT_MODE_LIMIT * BLOCK_SIZE:
            return 0
        else:
            return 1

class Sector:
    def __init__(self):
        self.data = [0 for _ in range(512)]
    
    def fill_data(self, data):
        if len(data) == 512:
            self.data = data
        else:
            for i in range(len(data)):
                self.data[i] = data[i]
class Lba:
    def __init__(self):
        self.data = []
        self.available = []
        for _ in range(LBA_SIZE):
            self.data.append(Sector())
            self.available.append(True)
    
    def get_available(self):
        for i in range(LBA_SIZE):
            if self.available[i]:
                return i
        return -1
class Ustar:
    def __init__(self):
        self.data = []
        self.available = []
        for _ in range(LBA_NUMBER):
            self.data.append(Lba())
            self.available.append(True)
    
    def get_available(self):
        for i in range(LBA_NUMBER):
            res = self.data[i].get_available()
            if res != -1:
                return (i, res)
    