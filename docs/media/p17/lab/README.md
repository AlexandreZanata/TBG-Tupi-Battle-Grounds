# P17 Stronghold kind matrix (D-46)

`tbg-balance strongholds --a <policy> --pairs 512 --threads 12` per
folder: row kind vs column kind, 512 seed pairs (1 024 seat-swapped games)
per cell, 25 600 games per policy, about 15–20 s each. A mirror cell (X vs
X) reads 52–54 %, not 50 %, so compare a cell with its row's mirror or
with the `Classic` row against the same column. The `bot` policy
fortifies a player kind (never `Classic`); `heavy`, `saver` and `evolver`
never fortify, so they measure the kinds' bases.

## Round 1 (product ratios: Artillery −1 100 HP / +2 300 damage bp, Bastion −1 500 damage bp)

- `evolver`: Artillery 62.2 / 66.6 / 63.4 / 61.1 % against Treasury,
  Bastion, Command and Classic; Bastion 38.4 % against Artillery.
- `bot`: Artillery 56.1 / 58.3 / 57.7 % against Treasury, Bastion, Command
  (its mirror 52.1 %).
- `heavy`, `saver`: every kind within the noise of its mirror.

## Round 2 (Artillery −1 500 / +1 500, Bastion −1 000)

- `evolver`: Artillery 59.0 / 61.4 / 61.7 %; Bastion 45.9 % against
  Artillery.

## Final (Artillery −1 500 / +1 000, Bastion −1 000): the files here

- `evolver`: Artillery 58.8 / 60.4 / 60.0 % against Treasury, Bastion,
  Command; the `Classic` row scores 56.6 / 58.2 / 56.2 % against the same
  columns, so Artillery is about +2.7 pp over the reference tower, inside
  one interval (about ±4 pp). Bastion 46.5 % against Artillery.
- `bot`: every kind is 2–7 pp under `Classic` against the same column:
  the bot spends 7 Energy on Fortification that `Classic` (which never
  fortifies) puts into units. That measures the bot's Fortify heuristic,
  not a branch's value; isolating each branch needs a fortify-vs-hold
  policy pair (open, P17 report).
- `heavy`, `saver`: every kind within the noise of its mirror.
