---
source_pdf: h:\OpenMatrix9\manuals\matrix8_book_1.pdf
source_key: m8book1
page_range: 91-92
section_title: Export Selected ￼
section_id: m8book1-077-export-selected
---
# Export Selected ￼

Source: matrix8_book_1.pdf, pages 91-92

<!-- section m8book1-077-export-selected | Export Selected ￼ | m8book1 pages 91-92 -->

<!-- page 91 -->

## New
## Export Selected
Export Selected converts the file type of the
selected objects for use in other programs. Matrix
supports a variety of file formats.
Click “New” to create a new file and discard any
items that are currently open on-screen. The new
file will be titled, “Untitled.3dm”, until it is saved for
the first time. If there is currently a model on the
screen Matrix will ask you if you want to save the
file before creating the new file.
Where is this Command:
Access this command from File>Export
Selected
Where is this Command:
Access this command from File>New
Steps:
Steps:
• Select New from the File menu to create a new file.
• Select objects to export.
• Select Export Selected from the File menu.
• Select the desired file type from the drop-down list
of supported file types.
• Type a name for the file in the File Name box.
• Click the Save button.
•
Note: After New is selected, Matrix will offer the
option to save the model. Click Yes, No, or Cancel:
Yes prompts the Save function (also found in the File
menu). No clears the viewports without saving the
model. Cancel will stop the “New” function entirely.
Any hidden objects or objects on layer colors that
have been turned off will be lost unless the current
model is saved.
## Open
### Menu Options:
Save Small: Removes the Render Mesh from
the model creating a smaller file which is easer
to share via email. The Render Mesh will be
regenerated the next time the file is opened, but
as a result, the model will shade and render more
slowly the next time you open the file.
The “Open” command loads a previously saved file
into the viewport. Click this command to access
the file browser and navigate to the location
where the file is saved. Select the file you wish to
Open. If there is geometry on the screen, prior to
using the Open command, Matrix will prompt you
to Save the existing file.
Where is this Command:
Access this command from File>Open
Steps:
• Select Open from the File menu.
• Select the Matrix file (.3dm) you wish to open from
the file browser.
Save Geometry Only: Saves geometry objects
only and discards all layers, materials, properties,
notes, or unit settings. A new file is created, but it
does not become your active Matrix model.
Save Textures: Saves any associated applied
Texture Mapping material to the exported
object(s).
Note: After Open is selected, Matrix will offer the
option to Save the model. Click Yes, No, or Cancel:
Yes prompts the Save function (also found in the File
menu). No clears the viewports without saving the
model. Cancel will stop the “Open” function entirely.
Notice the files are filtered to display .3dm files. Only
Matrix files may be opened using this method.

<!-- page 92 -->

or Save Small As, instead.
Where is this Command:
Access this command from File>Save
Note: Some file types have additional options that
can be accessed from the option button that appears
when one of these file types is selected. When a
file type creates mesh objects, the Polygon Mesh
Options dialog appears and can be used to specify
the mesh settings.
Steps:
• Select Save from the File menu.
• Specify a name for the .3DM file in the Save Matrix
Model dialog box.
• Click the Save button.
## Save As
## Import
Import allows the user to add an existing
geometry to the current project. Import can be
used with .3dm files (the native file format of
Matrix and Rhino) and any of the supported Matrix
file formats. The advantage of Import is that a
saved geometry can be added to an open file
without closing the current file.
Using the Save As function always prompts for a
file name. To overwrite an existing file (for example
if you wish to update a project), navigate to the
location where the files is saved and click on that
filename. The name of the file will appear in the
File name text box. Click Save. It is better to use
the Save As command, even when saving the
same model repeatedly, rather than using the
Save command. This ensures that a model is not
inadvertently saved over another file.
Where is this Command:
Access this command from File>Import
Where is this Command:
Access this command from File>Save
Steps:
Steps:
• Select Import from the File menu.
• Navigate to the desired file using the file browser
and click the Open button.
• Select Save As from the File menu.
• Specify a name for the .3DM file in the Save Matrix
Model dialog box.
• Click on the Save button.
Note: Matrix imports objects to the file on their
original layers and creates the layers if they do not
already exist.
Note: Save the file at different stages along the
design process so that if changes are required,
instead of starting the model again from scratch, it
can be revisited and amended as necessary.
If the imported file type does not contain layer
information, such as when you Save a file using the
Geometry Only option, the objects are placed on the
current layer.
## Save Small
## Save
The Save button prompts the user for a file name
if the project has not yet been saved. After a
project has been saved for the first time, the Save
function automatically saves over the current file
name for that project. This can be dangerous if
an existing file is open and the Save command is
used, because it will save over the previous model.
To avoid saving over another project, use Save As
The Save Small button functions just like the
Save button however, it removes the render
mesh or “skin” that is created when the project is
rendered or shaded. This render or polygon mesh
is regenerated and saved with the model the
first time it is shaded, making each subsequent
shading and rendering faster. To reduce the
file size, this mesh is not saved along with the
model during the Save Small operation. Use this
command in order to save disk space or to send
