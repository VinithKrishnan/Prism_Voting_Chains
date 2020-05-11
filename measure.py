import re
import sys


print(sys.argv)

logfile = sys.argv[1]

pattern = re.compile("^Received trans hash")
recv_time_map = {}

with open(logfile, 'r') as f:
    for line in f:
    # lines = f.readlines()
    # for line in lines:
        if pattern.match(line):
            separated = line.split()
            tx_hash = separated[3]
            recv_time = int(separated[-1])
            # extract the hash and time 
            recv_time_map[tx_hash] = recv_time 


pattern = re.compile("^Confirmed trans hash")
confirm_time_map = {}

with open(logfile, 'r') as f:
    # lines = f.readlines()
    for line in f:
        if pattern.match(line):
            separated = line.split()
            tx_hash = separated[3]
            confirm_time = int(separated[-1])
            # extract the hash and time 
            confirm_time_map[tx_hash] = confirm_time

for tx_hash in recv_time_map:
    if tx_hash in confirm_time_map:
        delay =  confirm_time_map[tx_hash] - recv_time_map[tx_hash]
        print(tx_hash, delay)