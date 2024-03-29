#!/Library/Frameworks/Python.framework/Versions/3.9/bin/python3

from diskStructures import *
import os

# Path to filesystem directory
directory = "./filesystem/"
disk_img = "./disk/disk2.img"

def construct_filesystem_tree(path):
    files = []
    name = "root" # give a name
    for child in os.listdir(path):
        print(child)
        child_path = os.path.join(path, child)
        if os.path.isfile(child_path):
            data = open(child_path, "rb")
            files.append(File(list(data.read()), child))
            data.close()
        else:
            # if the child is a directory
            subtree = construct_filesystem_tree(child_path)
            subtree.header.name = child
            files.append(subtree)
    return Dir(files, name)

# write the list of integers into the file
def write_ustar(data, disk_img_path):
    file = open(disk_img_path, "wb")
    binary_data = bytearray(data)
    file.write(binary_data)
    file.close() 

# Main function
def build_filesystem(fs_path, disk_img_path):
    tree = construct_filesystem_tree(fs_path)
    build_ustar(tree)
    data = USTAR.get_data()
    print(sum(data), 512 * 32)
    write_ustar(data, disk_img_path)

if __name__ == "__main__":
    build_filesystem(directory, disk_img)
