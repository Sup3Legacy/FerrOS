#!/Library/Frameworks/Python.framework/Versions/3.9/bin/python3
disque = open("disk.img", "w")

n = 400
l1 = [0]*512
l2 = []
l2 = l2 + [0]*(512-len(l2))
print('a'*512*n ,end="", file = disque)
disque.close()
