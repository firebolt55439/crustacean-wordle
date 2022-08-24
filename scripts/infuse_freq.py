# Usage: Pass dataset file (first column has word, last column has integer score, space-separated) and non-infused
#        file (a list of words). Outputs an "infused" version of the second file for which each word and
#        associated score appear on the same line, space-separated.
import os
import sys

infuse_from, infuse_to = sys.argv[1:]

score_map = {}
with open(infuse_from, "r") as fp:
    for line in fp:
        arr = [item for item in line.split() if item]
        word, score = arr[0], arr[-1]
        score_map[word] = int(score)

keeping = set()
with open(infuse_to, "r") as fp:
    for line in fp:
        word = line.strip()
        keeping.add(word)

infused = []
for word in keeping:
    assert word in score_map, f"'{word}' not found in dataset!"
    infused.append((word, score_map[word]))

infused = sorted(infused, key=lambda item: item[1], reverse=True)
for word, score in infused:
    print(word, score)
