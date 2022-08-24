import os, sys

for line in sys.stdin:
	arr = line.split()
	if len(arr[0]) == 5:
		print(line, end="")

