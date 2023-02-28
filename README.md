# Katalon engine
A super efficient katalon engine which can solve any state within a minute, usually in a few hundred milliseconds. This is achieved by implementing an efficient search algorithm. The techniques implemented here include: negamax (better known as minmax), alpha-beta pruning, move ordering, transposition table, symmetry generation, the killer heuristic and MTD(f) a form of null window search.

This project also includes a benchmarker and an interactive terminal interface. In the future it might also implement an openingstable, for which the first steps are already done.

_If_ the algorithms are all implemented correctly katalon is a win for the first player in at most 28 moves.

```
+-----------+---+-----------+
| .       . |   | .       . |
|           |   |           |
|     .     |   |     .     |
|       +---+---+---+       |
| .     | . |   | . |     . |
+-------+---+   +---+-------+
|       |     .     |       |
+-------+---+   +---+-------+
| .     | . |   | . |     . |
|       +---+---+---+       |
|     .     |   |     .     |
|           |   |           |
| .       . |   | .       . |
+-----------+---+-----------+
X > h
[0-4]<0-4>: make move
u undo: undo last move
e eval [timeout]: evaluate state
b best [timeout]: make best move
r random: make random move
n new: new game
l load: load game
c count: print movecount
t take: print takestreak
s square: print square
p print: print board
q quit: quit the maker
h help: show this help
X >
```
