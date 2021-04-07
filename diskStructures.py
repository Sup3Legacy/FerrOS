SHORT_MODE_LIMIT = 100
BLOCK_SIZE = 512  # in bytes
LBA_SIZE = 510

LBA_NUMBER = 32


class UstarException(Exception):
    def __init__(self, message="Error in ustar"):
        self.message = message
        super().__init__(self.message)


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
        return len(self.data)  # in bytes

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

    def add_file(self, file: UstarFile):
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
        return len(self.files) * 32  # in bytes

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
    
    def get_data(self):
        return self.data[:]

class Lba:
    def __init__(self):
        self.data = []
        self.available = []
        for _ in range(LBA_SIZE):
            self.data.append(Sector())
            self.available.append(True)

    def mark_unavailable(self):
        for i in range(LBA_SIZE):
            if self.available[i]:
                return i
        return -1

    def mark_available(self, i):
        self.available[i] = False
    
    def get_lba_table(self):
        # length : 512 bytes
        # The double 0 represents the index. Not used here
        data = [0, 0]
        for i in range(510):
            if self.available[i]:
                # TODO check the value that must be taken by True and False
                data.append(1)
            else:
                data.append(0)
        return data
    
    def get_data(self):
        data = self.get_lba_table()
        for e in self.data:
            data += e.get_data()
        # Because we only use 510 sectors from each lba
        data += [0 for _ in range(512)]
        return data

class Ustar:
    def __init__(self):
        self.data = []
        self.available = []
        for _ in range(LBA_NUMBER):
            self.data.append(Lba())
            self.available.append(True)

    def get_available(self):
        for i in range(LBA_NUMBER):
            if self.available[i]:
                res = self.data[i].get_available()
                if res != -1:
                    return (i, res)
                else:
                    self.available[i] = False
        return (-1, -1)

    def mark_unavailable(self, i):
        self.available[i] = False

    # Warning : this allocates the returned address!
    def get_address(self):
        (lba, sector) = self.get_available()
        if lba == -1 or sector == -1:
            print("Oops, got a problem while getting a fresh address")
            raise UstarException("Ustar struct could not allocate address.")
        else:
            self.mark_unavailable(lba)
            self.data[lba].mark_unavailable(sector)
            Address(lba, sector)
    
    def get_data(self):
        data = []
        for lba in self.data:
            data += lba.get_data()
        return data

# Glboal ustar object
USTAR = Ustar()