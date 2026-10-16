# Draft

![Draft of 0°, 8° and 16°: the bottom face stays put, the walls spread upwards.](img/part-draft/)

A cast or moulded part will not come out of the mould without draft: the walls must spread slightly towards the
parting line. Typical values are half a degree to three, depending on the material and the height of the wall.

## How to do it

1. Press **Draft** and click the **faces to tilt** on the body.
2. Press **Neutral face** in the bar above and click the face that stays in place: the tilt is measured from it.
   Usually it is the parting plane of the mould.
3. Type the **Angle** at the geometry — -60 to 60°; a negative angle tilts the other way. **Enter** applies,
   **Esc** cancels.

## Order

Draft goes **before** fillets: a fillet on a tilted face comes out right, a draft on a rounded one hardly will.

## If it did not work

- The angle is not taken — zero, above 60° or not a number; the reason is written at the field.
- The draft is not built — the reason is written at the field and on the timeline row. Most often it helps to put the
  draft before the fillet or to choose another neutral face.
