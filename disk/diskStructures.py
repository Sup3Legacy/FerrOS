SHORT_MODE_LIMIT = 100
BLOCK_SIZE = 512  # in bytes
LBA_SIZE = 510
LBA_NUMBER = 32


def perm(l):
    l2 = []
    assert(len(l)%2 == 0)
    for i in range(len(l)//2):
        l2.append(l[2*i+1])
        l2.append(l[2*i])
    return l2
class UstarException(Exception):
    def __init__(self, message="Error in ustar"):
        self.message = message
        super().__init__(self.message)


class Address:
    def __init__(self, lba=-1, index=-1):
        self.lba = lba
        self.index = index
        
    def part(self, i):
        if i < 2:
            return (self.lba >> (8 * (1-i))) & 255
        else:
            return ((self.index + 1) >> (8 * (3-i))) & 255
        


class Header:
    def __init__(self, name, type_, size):
        self.name = name
        self.type = type_
        self.block_addresses = []
        self.size = size
    
    def set_block_addresses(self, addresses):
        assert len(addresses) <= 100
        self.block_addresses = []
        for add in addresses:
            for j in range(4):
                self.block_addresses.append(add.part(j))
        self.nb_bloc = len(addresses)
        for _ in range(4*SHORT_MODE_LIMIT - len(self.block_addresses)):
            self.block_addresses.append(0)


    def get_address(self):
        return self.address
    
    def get_data(self):
        data = [0 for i in range(512)]
        data[ 0: 8] = [0, 0, 0, 0, 0, 0, 0, 0] # user
        data[ 8:16] = [0, 0, 0, 0, 0, 0, 0, 0] # owner
        data[16:24] = [0, 0, 0, 0, 0, 0, 0, 0] # group
        data[24:28] = [self.parent.part(1), self.parent.part(0), self.parent.part(3), self.parent.part(2)] # parent address
        data[28:32] = [(self.size >> (i * 8)) & 255 for i in range(0, 4)] # length
        data[32:36] = [(self.nb_bloc >> (i * 8)) & 255 for i in range(0, 4)] # nb blocs
        data[36:36+SHORT_MODE_LIMIT*4] = perm(self.block_addresses)
        assert(SHORT_MODE_LIMIT == 100)
        data[436:438] = [0, 0] # flags
        data[438:439] = [int(self.nb_bloc > SHORT_MODE_LIMIT)] # mode
        data[439:471] = [ord(i) for i in self.name] + [0] * (32 - len(self.name)) # name
        data[471:472] = [self.type] # file type
        data[472:512] = [0 for i in range(40)] # padding
        return data


class UstarFile:
    def __init__(self):
        None


class File(UstarFile):
    def __init__(self, data, name):
        self.header = Header(name, 2, len(data))
        while len(data) % 512:
            data.append(0)
        self.data = data

    def __len__(self):
        return len(self.data)  # in bytes

    def get_data(self):
        print(self.header.name, sum(self.data))
        return perm(self.data)

    def mode(self):
        if len(self) < SHORT_MODE_LIMIT * BLOCK_SIZE:
            return 0
        else:
            return 1


class Dir(UstarFile):
    def __init__(self, data, name):
        self.header = Header(name, 1, len(data))
        self.files = data

    def add_file(self, file: UstarFile):
        self.files.append(file)

    def get_data(self):
        data = []
        # get data segment of the dir
        for e in self.files:
            header = e.header
            name = header.name
            len_name = len(name)
            address = header.get_address()
            for i in range(28):
                if i < len_name:
                    data.append(ord(name[i]))
                else:
                    data.append(0)
            data.append(address.part(0))
            data.append(address.part(1))
            data.append(address.part(2))
            data.append(address.part(3))
            
        return perm(data)

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
        assert(len(self.data) == 512)
        assert(len(data) <= 512)
        if len(data) == 512:
            self.data = data
        else:
            for i in range(len(data)):
                self.data[i] = data[i]
            # TODO should maybe make sure the rest is 0
    
    def get_data(self):
        assert(len(self.data) == 512)
        return self.data[:]

class Lba:
    def __init__(self):
        self.data = []
        self.available = []
        for _ in range(LBA_SIZE):
            self.data.append(Sector())
            self.available.append(True)

    def mark_unavailable(self, sector):
        self.available[sector] = False

    def get_available(self):
        for i in range(510):
            if self.available[i]:
                self.available[i] = False
                return i
        return -1

    def mark_available(self, i):
        self.available[i] = False
    
    def get_lba_table(self):
        # length : 512 bytes
        # The double 0 represents the index. Not used here
        data = [0, 0]
        for i in range(255):
            if self.available[2*i] and self.available[2*i + 1]:
                # TODO check the value that must be taken by True and False
                data.append(1)
                data.append(1)
            else:
                data.append(0)
                data.append(0)
        return data
    
    def get_data(self):
        data = self.get_lba_table()
        assert(len(data) == 512)
        for e in self.data:
            data += e.get_data()
            assert(len(data)%512 == 0)
        # Because we only use 510 sectors from each lba
        data += [0 for _ in range(512)]
        assert(len(data) == 512 **2)
        return data
    
    def set_sector_data(self, i, data):
        self.data[i].fill_data(data)

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
                    # print("given :", (i, res))
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
            #self.mark_unavailable(lba)
            # This is useless
            #self.data[lba].mark_unavailable(sector)
            return Address(lba, sector)
    
    def get_data(self):
        data = []
        for lba in self.data:
            d = lba.get_data()
            data += d
        return data
    
    def set_sector_data(self, lba, sector, data):
        self.data[lba].set_sector_data(sector, data)

# Glboal ustar object
USTAR = Ustar()

def build_ustar(tree, parent = Address(0, 1)):
    # tree is suposed to be a Dir containing the file hierarchy
    if isinstance(tree, File):
        length = len(tree) # number of children
        sector_number = length // 512
        if length % 512 > 0:
            sector_number += 1
        mode = tree.mode()
        # Length should be coherent with the mode
        if mode == 0:
            # short-mode
            addresses = [USTAR.get_address() for _ in range(sector_number + 1)]
            # First address if for the header
            tree.header.address = addresses[0]
            tree.header.parent = parent
            # Set all addresses from blocks into the header
            tree.header.set_block_addresses(addresses[1:])
            # Put the header in the sector
            USTAR.set_sector_data(addresses[0].lba, addresses[0].index, tree.header.get_data())
            # Put data into the sectors
            file_data = tree.get_data()
            print(len(file_data))
            for i in range(sector_number):
                current_add = addresses[i + 1]
                print(tree.header.name, i,sum(file_data[i*512:(i+1)*512]))
                USTAR.set_sector_data(current_add.lba, current_add.index, file_data[i*512:(i+1)*512])
            # return the address of the file header
            return addresses[0]
        elif mode == 1:
            # Long-mode
            # We first need to allocate additionnal blocks
            supersector_numer = sector_number // 128
            if sector_number % 128 > 0:
                supersector_numer += 1
            # Get all needed addresses
            header_address = USTAR.get_address()
            supersector_addresses = [USTAR.get_address() for _ in range(supersector_numer)]
            addresses = [USTAR.get_address() for _ in range(sector_number)]
            # Input all addresses into header
            tree.header.set_block_addresses(supersector_addresses)
            # Write the header
            USTAR.set_sector_data(header_address.lba, header_address.index, tree.header.get_data())
            # Write all superblocks
            for i in range(supersector_numer):
                # data to put into the superblock
                superdata = addresses[i * 128: (i + 1) * 128]
                partial_len = len(superdata)
                # TODO remove this
                for i in range(128-partial_len):
                    superdata.append(Address(0, 0))
                assert len(superdata) == 0

                super_address = supersector_addresses[i]
                # Write the superblock
                USTAR.set_sector_data(super_address.lba, super_address.index,superdata)
            tree.header.address = header_address
            tree.header.parent = parent
            file_data = tree.get_data()
            # Write all blocks
            for i in range(sector_number):
                current_add = addresses[i]
                USTAR.set_sector_data(current_add.lba, current_add.index, file_data[i*512:(i+1)*512])
            return header_address
        else:
            raise UstarException("Undefined mode")
    elif isinstance(tree, Dir):
        print("added dir")
        length = len(tree)
        sector_number = length >> 9 # We can put 16 children per sector
        if length % 512 > 0:
            sector_number += 1
        mode = tree.mode()
        print("Dir ", tree.header.name, " nb_bloc ", sector_number)
        if mode == 0:
            header_address = USTAR.get_address()
            block_addresses = [USTAR.get_address() for _ in range(sector_number)]
            tree.header.address = header_address
            tree.header.parent = parent
            tree.header.set_block_addresses(block_addresses[:])

            children = []
            for e in tree.files:
                name = e.header.name
                address = build_ustar(e)
                children.append((name, address))
                
            header_data = tree.header.get_data()
            assert(len(header_data) == 512)
            USTAR.set_sector_data(header_address.lba, header_address.index, header_data)
            dir_data = tree.get_data()
            print("Header data", dir_data)
            for i in range(sector_number):
                current_add = block_addresses[i]
                print(current_add.lba, current_add.index)
                USTAR.set_sector_data(current_add.lba, current_add.index, dir_data[i*512:(i+1)*512])
            print("Header nb_bloc ", int(tree.header.nb_bloc > SHORT_MODE_LIMIT))
            return header_address

        elif mode == 1:
            print("long")
            None
        else:
            raise UstarException("Undefined mode")
    else:
        raise UstarException("Unrecognized type in tree")