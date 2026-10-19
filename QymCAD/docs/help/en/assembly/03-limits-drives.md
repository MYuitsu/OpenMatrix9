# Joint limits and drives

A movable joint — a hinge, a slider, a cylindrical, planar or ball joint, a pin in a slot — leaves the part some
freedom. You can set it to a number (a drive) or bound it (limits).

![The same mechanism driven to 0°, 40° and 80°: the arm moves by the rule, not by hand.](img/assembly-drive/)

## Where it is set

Click the joint — its glyph in the view or its row in the **Joints** list. The fields of its degrees of freedom
stand in the properties on the right and in the popup at the glyph; they are the same fields.

## Drive: put the part in a position

* The field of a degree — an angle or an offset — shows where the part is now. Type a number or drag the value: the
  degree becomes a drive, the part moves to that position and stays there. The lock beside the field closes.
* The field takes a formula over the document's parameters: `pa`, `pa/2`, `90-pa`. Tie several drives to one
  parameter, and the whole mechanism goes through its travel when you change one number in the **Parameters**
  window.
* To release the degree, press the lock: the part is free again, and you can lead it with the mouse.

## Limits: bound the travel

* Open **Limits**. For each free degree tick **min** and/or **max** and set the value.
* The part does not go past a limit, neither when you lead it nor on a recompute. At the limit the degree reads
  **at the stop** — that is the bound you set, not a fault.
* A lid that opens to 200° in the model will hit the housing in real life: a limit of 110° shows that at once.

## If it did not work

* The part does not move when you lead it — the degree has a drive. Press the lock to release it.
* A red line under the joint says the mate is not satisfied — two joints ask different things of the part. Remove
  one of them or release a drive.
