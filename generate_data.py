from random import shuffle

freqs = {
  "E" : 120,
  "D" : 42,
  "L" : 42,
  "U" : 37,
  "C" : 32,
  "M" : 24,
  "K" : 7,
  "Z" : 2,
}

s = ""
for k,v in freqs.items():
  chars = k * v
  s = s + chars

arr = list(s)
shuffle(arr)

message = "".join(arr)

with open("./data/indiana.txt", "w") as f:
  f.write(message)