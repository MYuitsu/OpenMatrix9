---
source_pdf: h:\OpenMatrix9\manuals\matrix8_book_1.pdf
source_label: Matrix 8 Instruction Manual Book 1
page_range: 48-53
section_title: History
section_id: m8book1-031-history
shared_start_page: False
shared_end_page: False
---

# History

Source: matrix8_book_1.pdf, pages 48-53

<!-- section m8book1-031-history | History | m8book1 pages 48-53 -->

<!-- page 48 -->

# CHAPTER 8
# History
### In Matrix, History enabled commands can be used to your advantage to create
### flexible models. Below is an overview of the Rhino and Matrix History functions, how
### they work, and what best practices to keep in mind when using these powerful tools.
Rhino History “On”
To begin, first make sure that Rhino History is turned
on. The button is located in the Info & Settings
menu. This should default to the ‘ON’ position, but
double check its status.
Rhino History Enabled Commands
To view the commands that contain Rhino
History, press the F1 key. This will activate the
Rhino Help menu.  When the window appears,
type in “History.”  A brief description of History will
display, along with a full list of all the commands
that are History enabled. This list has been greatly
expanded, increasing your modeling possibilities.
Note: There may be times when you do not wish for
History to be active.  If so, simply click the tool OFF
before creating a “child” object.

<!-- page 49 -->

Broken History
The relationship is broken if a Child object is
altered apart from the Parent. When this occurs,
a warning message window will appear. If it was
not your intention to break History between the
Parent and Child Objects simply “Undo” the last
action by clicking on the Undo button in the Main
Menu or pressing ‘CTRL + Z’ on your keyboard.
Then, repeat the desired action on the Parent
object instead of the Child.
The Relationship
When working with Rhino History, it all begins
with an original object or objects. History stores
information about these objects (or geometry)
and when this object is duplicated, mirrored, or
used in combination to create new objects, a
relationship is established between the original
object, (known as the PARENT), and the new
object, (known as the CHILD). When the original
or Parent is updated, the Child follows.
There are certain commands that will break
History and this is unavoidable. Commands such
as Join and Boolean change the Child geometry,
be it a curve or a surface, so that it no longer has
a Parent/Child relationship with the geometry that
created it. It is recommended that you Job Bag,
or save a design, before performing one of these
commands. That way you can revert back to your
flexible history-enabled model if necessary.
Move and Scale the original sphere (Parent)
and the Mirrored sphere will update to match.
Note: Sometimes a Rhino update will also break
History. So, saved models that had History may lose
it after installing an update.
Disabling the “Broken History” Warning
To disable the History Warning text box, simply
type the word ‘History’ into the Command Line.
The options menu
offers to set the
“Broken History Warning” to “NO.”
For information on the other Command Line
options for History, see the History command in
the Info and Settings section of the Introduction.
CAUTION:  Be aware that without the Broken History
warning, it will not be clear when a relationship
has “changed” between objects. It may not be
discovered for several steps and can be a challenge
to track, after the fact. It is recommended that
you give yourself time, and experience, before
deactivating this message.

<!-- page 50 -->

Color-Coding
When working with History, color-coding the
Parent, separate from the Children, can help
ensure that changes are made to the correct
object, thereby, preserving the relationship.
Strategize
When working with History, color-coding the
Parent separate from the Children can help ensure
that changes are made to the correct object,
thereby, preserving the relationship.
When a strong history tree is established,
the modeling process can be extremely fluid.
Changes can occur to your surfaces with just a
small adjustment to a single curve, as seen below.
Simple
In this simple example (above), the Parent object
is green and the children are gray.
Complex
In this design (below), there are multiple Parent/
Child relationships at work.
Angle and position of curves used as part of a blend
were moved, updating the Blended Curve, the Pipes
on those curves, and the Polar-arrayed Pipes forming
the wires.
Color-coding, for this example ONLY, each
Parent has been color-coded “red” for ease of
identification.
Layers of History
History can actually be layered with multiple
“generations,” if you will.  In the set up of a model
with Rhino History, there will be Parent objects
and Child objects; but, keep in mind that Child
objects can produce Children as well. When the
Parent is updated, the Child and Grandchildren
update too. If the second generation is updated, it
will break the relationship with the Parent, but its
Children will continue to update and follow. This all
needs to be tracked as you model.

<!-- page 51 -->

When coloring your own Parent/Child objects,
a good rule of thumb is to leave Parents their
standard layer colors and re-color children.  Layer
04 Gray works well for this purpose.
one gem is manipulated in these tools, the effect
can be stunning: causing all gems - for example,
a Pavé’ layout - to flow like a school of fish toward
one curve or point on-screen.
### Matrix History
Setting Tools that Follow their Gems
Thanks to Matrix History, there are a number of
settings tools that follow their gems around like
lovesick puppies. Move a gem with the MSR
(Move, Scale, and Rotate) handles (aka Gem
Positioner in the gems menu), or just pick it up
and drag it around, and bezels, prongs, Metal
Piece (metal built around a line of gems), and
channel border (a rectangle curve built around a
line of gems) will follow. The powerful thing about
prongs, Metal from Gems, and Channel Border is
that these tools are built around a line of gems, so
the entire layout - in the case of Metal Piece, an
entire surface - can be changed as gems are
scaled, rotated, and manipulated, making History a
design tool in itself: when you want to adjust one
of these items, simply make changes to the gems
and the surfaces follow.
A Gem Never Forgets
In addition to the powerful “Edit (Object)” or “MSR”
found in the F6 Menus, Matrix History is achieved
above all through the gems in the program. Gems
retain a memory of most of the program’s Settings
and Cutters Tools that were created on them
(including Heads, Bezels, Bezel Cutter, Prongs,
Metal Piece, Channel Border, Azure Cutter,
Channel Cutter, MicroProng Cutter, Bright Cutter,
Bright Cut Channel; excluding Prong on Surface /
Bead on Surface, Emerald Profile & Cluster,
Millgrain, & Gem Cutter Library). Using the “Match
Attributes” a Setting or Cutter currently on a gem
or ever created on a gem (even if it was deleted)
can be re-applied to the same gem or to another
gem in the program. When that object appears, it
can be edited by selecting F6 and the original
builder used (where available). Alternately, without
running Match Attributes, simply input the gem
that already had the object on it into the same
builder, and the last settings used in that builder
will appear on the gem.
Move or adjust the rotation or even scale a gem and the
history enabled setting built on it will update.
Gem Control: When either sphere is moved the the gems
north or culet direction will move to point to the control
object.
Gem Follow and Gem Control, both of which
assign the gems a “North” axis that can be told to
point at another gem (Gem Follow) or any object
on the screen (Gem Control). When more than

<!-- page 52 -->

History Update”, and they must both be on to see
the changes History is making while you work. If
“Matrix History Update” is turned off but “Record”
is on, you will not see the changes, but History will
be making them “behind the scenes”. When you
turn on “Update” again, they will be visible. In this
way, you can make changes without your
computer having to “chug” away to keep up with
Matrix History
enabled Commands
have a yellow arrow
in the corner as seen
here in Profile Sweep
Other Tools with Matrix History
In this manual, you’ll learn to use lots of Matrix
tools that are enabled with History. Starting in the
Curve Menu, the Curve from 2
Views tool creates advanced
shapes that are interactive with
History; so that you can edit
them and see your changes
keep up in real time. The
Surface menu contains History
enabled Sweep commands
such as Sweep 1 History and
Sweep 2 History.
From left to right Matrix History Update, Matrix History
Record, Matrix Object History Clear
you. But usually, you’ll want them both on at the
same time to see the results while you work. If
both settings are turned Off, NONE of the History
functions described in this section will be available
so it is a good idea to always leave both settings
on.
Application of History
By building models with these interactive surface
tools, you are creating incredibly flexible models
that can be changed in subtle or dramatic ways to
accommodate your design changes. In this way,
you could use one design many times for many
different customers, changing it in subtle ways
(new gem size or shape, new cross-section shape
for shank) or dramatic ways (new setting type or
an entirely new ring shape) and the builders can
make the changes for you without the need to
rebuild the entire model from scratch, as you
would have had to do in previous versions of the
program.
The only reason you would want to turn them
off is to make major changes to your model: for
instance, when moving a ring created with Profile
Placer and Sweep History or a model created with
Metal from Gems clear across your screen - as
when setting it up for a render, for example - you
will find that all the pieces take a moment. To
avoid this convoluted look turn Off History before
attempting major moves like this.
Make minor or major changes to your design and watch it
update.
But don’t worry: none of the History relationships
are “broken” when you turn History off on a model
like this. You can turn it right back on and pick up
modeling where you left off. I like to think of it as
“suspending” History. If, however, you attempt to
build more models once it has been turned off,
History will not apply for them even after you turn
it back on. It must be on at the start of working
with a History-enabled tool. If however you are
completely done with the History on an object
and you want the program to forget the history
you can use the “Matrix Clear Object History”
command. This function is object specific and
allows you to pick one object at a time and “Clear
History”.
Turning History On and Off
History defaults to On when you open up the
program. In the Info and Settings menu in the
Matrix interface, you will see the History settings
on the far right under this menu’s title bar. Matrix
History appears like a “Record” and “Play” button.
These are “Matrix History Record” and “Matrix

<!-- page 53 -->

3- Don’t delete the input objects or the
output objects.
Obviously, if you delete the input objects used to
make the surface; or the output surface you are
working on, History will be broken.
What it does is, when you copy or re-create
an object using one of its tools, you can make
changes to the original object (sometimes flat
against the grid or easier to manipulate in some
other way) and the changes will be carried
through to the new object. This can be helpful
in that, if you have two objects that you want to
be identical and you edit one, the new object will
also reflect your changes. However, it can also be
a pain: what if you forgot you had it on, and you
wanted the new object to be left alone - NOT
exactly the same as the first.
### Tips & Tricks: Using History
These tips and tricks for not breaking apply to
both Matrix and Rhino History
1- For any History processes involving
curves - don’t Split, Trim, Join, or Curve
Boolean curves.
The thought here is: these processes create a
NEW curve, which is no longer part of the History
relationship. If you need to edit the curves using
these tools, do so before running the command;
or edit them and re-run the command. For join
you can use the Join History command which
creates a new curve that is a child of the unjoined
curves. Use the new curve to create new output
geometry with commands like Curve Network.
The parent curves can be updated and the joined
curve and surface with update.
2- For any History process that creates
surfaces - don’t Split or Cut away
(Boolean) objects from the surface.
This rule is similar to the previous one, only you’re
breaking it from the other end: not breaking the
input objects, but breaking the output objects.
Once the output object’s history relationship with
their curves are broken, changes to the curves
won’t continue to change the surfaces. However,
there is a way around this limitation: if you use
Matrix’s Profile Placer to create the shape curves,
you can always build the surface again with
Matrix’s Sweep History tools and re-cut it with the
Gem Cutter history available on gems (Match
Attributes of a gem or gem line to itself and select
the cutters), and you can re-sweep and re-cut the
surface.
