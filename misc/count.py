import math

def count(red, yellow, blank):
    return math.factorial(red + yellow + blank) / \
        (math.factorial(red) * math.factorial(yellow) * math.factorial(blank))

total = 0

for red in range(0, 13):
    for yellow in range(0, min(12, 21 - red) + 1):
        blank = 21 - red - yellow
        total += count(red, yellow, blank)

print(total)
