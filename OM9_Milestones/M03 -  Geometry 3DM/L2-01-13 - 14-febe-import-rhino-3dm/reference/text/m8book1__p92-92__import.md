# Import

- Source PDF: h:\OpenMatrix9\manuals\matrix8_book_1.pdf (Matrix 8 Instruction Manual Book 1)
- Page range: 92-92
- Section title: Import
- Section id: m8book1-078-import
- Kind: command
- Note: boundary page(s) are shared with neighbouring sections (two-column layout); text/images on those pages may belong to the neighbour.
- Images (../images/): m8book1_p92_img01.png

---
<!-- section m8book1-078-import | Import | m8book1 pages 92-92 -->

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
