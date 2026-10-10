# Third-party notices

OpenMatrix9 uses FreeCAD's public APIs and inherits the project's existing
LGPL-2.1-or-later declaration. FreeCAD is maintained by the FreeCAD contributors:
https://github.com/FreeCAD/FreeCAD. Preserve copyright and license notices when
copying or modifying any upstream code.

FreeCAD and its SDK bring their own dependencies, including Qt, Open CASCADE,
Coin, Python and others. Release 0.0.1 provides a separate matching FreeCAD portable host. Its component
licenses and package metadata are included in that archive; FreeCAD source is
provided as a separate release asset. Their respective licenses apply.

Release 0.0.2 distributes only the OM9 workbench for the separately installed
official FreeCAD 1.1.4 host. The host and its DLLs are not included in that
plugin archive. The stock selection adapter derives from FreeCAD 1.1.4's
LGPL-2.1-or-later CommandView implementation, with attribution retained in source.

3DM exchange uses the open-source openNURBS toolkit from Robert McNeel &
Associates, fetched at the pinned revision in `cmake/OpenNURBS.cmake`:
https://github.com/mcneel/opennurbs. Its license and bundled zlib notice are in
`Resources/licenses/`. Do not remove those notices or confuse openNURBS with a
commercial Rhino SDK.

All distributed icon SVG geometry is authored for OpenMatrix9. Palette choices
and functional cues were informed by historical product references. No
extracted icon pixels, product logos, commercial binaries or private Matrix
manuals are distributed in this snapshot. Matrix, Gemvision, Rhino, Stuller and other product
names that remain as compatibility identifiers belong to their respective
owners. Such identifiers do not grant rights in vendor code or artwork.

The user-approved documentation exception in `ref/rhino5` contains the unchanged
Rhino 5 User's Guide (Windows), publicly downloadable from Robert McNeel &
Associates at
https://docs.mcneel.com/rhino/5/usersguide/en-us/windows_pdf_user_s_guide.pdf.
The original notice is “© Robert McNeel & Associates, 11/30/2016.”
Its original notices and applicable terms remain in effect; the project's
LGPL declaration does not relicense this vendor document. The source URL,
byte count, SHA256 and verified download identity are in `ref/rhino5/SOURCES.json`.
This exception does not cover recovered code, vendor SDKs, images or Matrix manuals.

The publication audit records technical exclusions and checks. It does not
establish ownership of every implementation detail or certify freedom from
copyright, trademark, contract or patent claims.
