#!/Library/Frameworks/Python.framework/Versions/3.9/bin/python3

from diskStructures import *
import os

# Path to filesystem directory
directory = "./filesystem/"
disque = open("disk.img", "w")

n = 4096
l1 = [0]*512
l2 = []
l2 = l2 + [0]*(512-len(l2))
print('a'*512*n ,end="", file = disque)
disque.close()

def construct_filesystem_tree(path):
    files = []
    name = ""
    for child in os.listdir(path):
        child_path = os.path.join(path, child)
        if os.path.isfile(child_path):
            name = child
            data = open(child_path, "rb")
            files.append(File(data, name))
            data.close()
        else:
            # if the child is a directory
            subtree = construct_filesystem_tree(child_path)
            subtree.header.name = child
            files.append(subtree)
    return Dir(files, name)


construct_filesystem_tree(directory)