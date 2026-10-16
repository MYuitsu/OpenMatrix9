# External references: top-down design

A sketch of one part can stand on a face of another. The parts then share an **external reference**: the sketch
follows the face wherever it goes.

This is how top-down design works: the housing comes first, and the bracket, the lid and the gasket take their sizes
from it. Retyping those sizes by hand means keeping a second copy of one value and one day editing only one of them.

## How to make one

1. Step into the part you are building (double-click it in the tree).
2. Turn on **In context** in the bar above: the neighbouring parts show as ghosts.
3. Press **Sketch** (the pencil, key K) and click a face of the neighbouring part — as you would your own.

The sketch stands on that face, and the reference is recorded by itself. Everything built on the sketch follows the
neighbour: move or edit the housing, and the bracket rebuilds.

## How to see and break it

Select the part in the tree: the properties on the right list, under **External references**, the faces of the
neighbours it stands on. The button with a cross beside a row is **Break the link**: the geometry freezes in place as
a snapshot and stops following the neighbour. Do this when the part goes into production and must no longer change
after the housing.

## If it did not work

* The neighbours are not shown — **In context** is off.
* The sketch does not follow the neighbour — the link was broken: the row is gone from the part's **External
  references**. Build the sketch on the face again.
* Do not make rings: if the housing takes its sizes from the bracket and the bracket from the housing, the pair does
  not rebuild. Keep the design one way — from the leading part to the following one.
