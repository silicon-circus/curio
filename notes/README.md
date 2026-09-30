# notes

**What mjanime delivered, and what became of it.** One file per subject.

These are not a record of what curio serves — two of them describe art that never entered curio at
all. They are the record of a *delivery*: what was asked for, what came back, what was wrong with it,
and where each file ended up. The part worth keeping is rarely the file list. It is the paragraph
explaining that naming a garment makes the generator draw the garment, or that explaining *why* inside
a prompt is indistinguishable from instruction. That knowledge is invisible in the output and
expensive to relearn.

## How art gets here

1. **@mjanime** (aka @minanime) generates prop art and drops it in the toplevel **`intake/`**, grouped
   by subject — `intake/blimp/`, `intake/honey-basket/` — with a `README.md` in the group describing
   what he made.
   *(That is `intake/` at the repo root, not `data/intake/`. It is gitignored, it is not the store's
   intake, and the console never looks at it.)*
2. His `README.md` is renamed to **`notes/<SUBJECT>.md`** and moved here. Its presence here means the
   group has been looked at; a `README.md` still sitting in `intake/<group>/` means it has not.
3. The good files are named properly — his choices are a starting point, not a decision — and routed:
   into curio's `data/names/`, or into a game repo that vendors its own art, or neither.
4. The rest stay in `intake/`. Nothing is deleted, so `intake/` is also the reject pile.

## Why some art skips curio entirely

Games that ship as store builds — Steam, mobile — cannot fetch an asset by name at runtime, so they
vendor it in their own repo. `bloom-static` and some of the `wcfranks` arcade games work this way.
Art for them passes through `intake/` and never reaches `data/names/`, which is correct and not an
oversight.

## Conventions

**Reference a file by the name it was delivered as, and record where it went.** Files get renamed
when they are filed, and sometimes renamed in place in `intake/` before that, so a note that only
carries the delivered names goes stale the moment a decision is made. The delivered name is the
historical fact; the disposition is what makes it findable. Hence the `## Disposition` section at the
foot of each note — add one when a group is routed, and do not rewrite the tables above it, because
those describe what actually arrived.

**Record what failed.** A rejected variant and the reason for rejecting it are worth more than another
line about the one that worked.

**Leave `**Why:**` blank rather than guessing.** Several of these were reconstructed by hashing bytes
long after the decisions were made. Hashes recover *where a file went*; they cannot recover why.

## Queue

A group whose `intake/` directory is empty is finished. A group with a `README.md` still in it is
waiting on naming decisions:

    arena           184 files   231 MB   README.md, README.seats.md, README.wedge.md, seatless/README.md
    holiday-icons    73 files    52 MB   README.md
    stopwatch         4 files   4.5 MB   README.md
    hinge             3 files   1.8 MB   README.md
