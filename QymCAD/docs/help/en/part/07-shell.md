# Shell

Key **H**.

![The shell took out the material inside, leaving walls of 2 mm; the top face was picked to be open.](img/part-shell.png)

The body becomes hollow: the material inside is removed, the walls get the given thickness, and the faces you pick
disappear — the hollow opens through them.

## How to do it

1. Press **Shell** and click the faces to **open** (one or several).
2. Type the **Thickness** at the geometry; in the bar above — where the wall goes: **Inwards**, **Outwards** or
   **Centred** (half each way).
3. **Enter** applies, **Esc** cancels.

## Order in the timeline

A shell goes **after** the main shape but **before** small fillets: then the fillets land on both the outer and the
inner edges.

## If it did not work

The thickness is not taken — in a narrow place such a wall does not exist: no material is left. Make the thickness
smaller; the reason is written at the field.

For an even wall along a flat contour rather than over the whole body, see [offset](sketch/20-offset).
