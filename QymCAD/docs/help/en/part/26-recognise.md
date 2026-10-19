# Recognise

Turns an imported mesh (STL, OBJ, 3MF and others) into a body you can work on: cut, drill, sketch on a
face. Click the mesh, choose what to get, **Enter**.

## How

1. Press **Recognise** in the Part bar.
2. Click the mesh in the viewport. A second click lets it go.
3. In the top bar choose what to get: **Exact surfaces** or **As it is (a polyhedron)**.
4. **Tolerance** - at the mesh, as a number or a formula. 1 suits a mesh exported from a CAD; a coarser
   mesh (a scan, a remeshed one) needs more - 10 or 100.
5. **Simplify, mm** - how far the mesh may move from itself to become lighter. A large mesh (from 50 000
   triangles) gets the field filled by itself - a ten-thousandth of its size - and the bar warns that the
   body would be heavy without it. 0 leaves the mesh as it is.
6. **Enter** applies; **Esc** cancels.

## What comes out

- **Exact surfaces** - planes, cylinders, cones, spheres, tori, the helical faces of a thread, an auger and a spring, and smooth free forms are found on the mesh and the body is built of them. The wall of a hole becomes a cylinder, not dozens of flat strips; a thread's flank becomes one helical face, a spring's wire one face.
- **As it is (a polyhedron)** - every flat group of triangles becomes a face. Quick and always works, but round stays faceted.

## Good to know

- **The mesh is consumed.** The tree keeps a "Recognised body" node and the mesh leaves the screen.
  Undo brings it back in one step.
- **A click on anything but a mesh** picks nothing and says so at once.
- **What fits no surface stays as pieces of mesh inside the body.** If there are many, raise the
  tolerance or take **As it is**.
- **The body is slow and takes long to rebuild** - the mesh is too fine: every triangle became a face.
  Undo the recognition and recognise again with **Simplify, mm** set - 0.01, say: there will be several
  times fewer triangles, and the shape moves no further than that.
- **Only a shell came out, not a body** - the mesh has holes or flipped triangles. Try a larger
  tolerance; if that does not help, **As it is** gives a polyhedron.
