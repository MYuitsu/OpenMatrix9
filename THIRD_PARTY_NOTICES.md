# Third-party notices

OpenMatrix9 uses FreeCAD's public APIs and inherits the project's existing
LGPL-2.1-or-later declaration. FreeCAD is maintained by the FreeCAD contributors:
https://github.com/FreeCAD/FreeCAD. Preserve copyright and license notices when
copying or modifying any upstream code.

FreeCAD and its SDK bring their own dependencies, including Qt, Open CASCADE,
Coin, Python and others. This repository does not redistribute their binaries;
their licenses apply when building or distributing a combined application.

3DM exchange uses the open-source openNURBS toolkit from Robert McNeel &
Associates, fetched at the pinned revision in `cmake/OpenNURBS.cmake`:
https://github.com/mcneel/opennurbs. Its license and bundled zlib notice are in
`Resources/licenses/`. Do not remove those notices or confuse openNURBS with a
commercial Rhino SDK.

All distributed icon SVG geometry is authored for OpenMatrix9. Palette choices
and functional cues were informed by historical product references. No
extracted icon pixels, product logos, commercial binaries or manuals are
distributed in this snapshot. Matrix, Gemvision, Rhino, Stuller and other product
names that remain as compatibility identifiers belong to their respective
owners. Such identifiers do not grant rights in vendor code or artwork.

The publication audit records technical exclusions and checks. It does not
establish ownership of every implementation detail or certify freedom from
copyright, trademark, contract or patent claims.
