#!/Library/Frameworks/Python.framework/Versions/3.9/bin/python3
disque = open("disk.disk", "w")

n = 100
l1 = 'a'*512
l2 = "je suis Samuel"
l2 = l2 + '-'*(512-len(l2))
print(l1+l2+'a'*n*512,end="", file = disque)
disque.close()
