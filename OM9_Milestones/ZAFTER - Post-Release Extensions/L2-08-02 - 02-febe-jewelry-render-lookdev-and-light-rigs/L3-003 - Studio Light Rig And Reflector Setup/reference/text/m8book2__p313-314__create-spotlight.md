# Create Spotlight

- Source PDF: Matrix 8 Instruction Manual Book 2 (matrix_8_book_2.pdf)
- Page range: 313-314
- Section title: Create Spotlight
- Section id: m8book2-252-create-spotlight
- Note: boundary page(s) are shared with a neighbouring section (two-column layout); text/images may include neighbour content.

---
<!-- section m8book2-252-create-spotlight | Create Spotlight | m8book2 pages 313-314 -->

<!-- page 313 -->

### Options:
toggle to the Bloom option (look where RGB
color is shown).
Size: The Size slider determines how blurry the
map is.
Base of Cone: The “Base of Cone” is the widest
part of the spotlight, or, the width that the light
will be around the item being rendered. Click on
a point or type grid coordinates (such as F4 on
which to center the base). The Command Line will
switch to “Radius”.
Glare Effect: The Glare Effect occurs as a post
render process. It appears after the
render is complete. Glare gives you metal
reflections that look very hot or bright. It’s like
a glint or a lens flare in Photoshop. This effect is
more noticeable when you have physical lights
added to the scene. Glare’s weight and size sliders
perform like Bloom’s do.
Radius: Either move the mouse until the base is
the correct size or type in a distance for the radius.
Please note that a very wide base tends to diffuse
into the surroundings quickly, so keep the base
narrow for a more direct effect; wide for a more
diffuse effect.
Diffraction: The Diffraction toggle produces
colored patterns in the glare.
End of Cone: The closer the end of cone is to
the base of the cone, the sharper the falloff or
focus of the light. For diffuse light, set it farther
away; for sharp light, set it closer.
Moving the Spotlight: To move it around, turn
on its Control Points and click and drag them to
change it.
## Create Spotlight
## ’
Notes:
The shape of the cone affects the lighting;
however, the cone for the spotlight represents the
direction of the light, not the range of the light.
Spotlights with narrower cones produce more
detail than spotlights with wider cones. In addition,
the closer the smaller circle is to the large outside
circle at the base of the cone, the sharper the
falloff or focus of the light.
‘Spotlight’
The Create Spotlight button creates a cone-
shaped spotlight object used for additional
lighting in rendering. Two commands help
you create and control a spotlight. The Create
Spotlight button creates a cone -shaped spotlight
object. The Spotlight object is projected in the
direction of the open cone. See Modeling with
Solids menu “Cone” command.
Where is this tool:
Access this tool from Render>Create Spotlight
Objects near the center axis line of the cone are
illuminated the most brightly. That bright, full
illumination continues for half of the width of the
cone, and then slowly drops off to nothing at the
very outside of the cone.
Steps:
• To use this command, start with the model to
render in the viewports.
• Select Create Spotlight from the Render menu.
• Set the point for the base of cone.
• Set the radius of the base of cone.
• Pick a point for the end of cone.
Setting Properties of the Spotlight: To
adjust the light intensity of any Light object
(not Emitters), select the light object and press
Ctrl + T (or type “Properties” into the Command
Line and press Enter). Use the Spotlight icon
menu near the top of the Properties tab to adjust
its configuration for different effects.

<!-- page 314 -->

Enabled: Enabled turns the light object on and
off.
Color: Color provides a tint to the spotlight.
Clicking here opens a color selection box.
object merely indicates the direction the light
shines.
• Turn on the light’s control points and move or drag
the control points to edit the light’s direction and
target.
• To move the light without changing its direction,
move the dolly point (the control point at the
midpoint on the direction vector).
Intensity: Intensity turns up (or down) the
brightness of the light.
Shadows: Shadows turns a shadow on or off.
## Set Spotlight to View
Shadow Radius: Shadow radius increases or
decreases the size of the shadow.
## Create Point Light
‘SetSpotLightToView’
Matches the view to a spotlight direction. Changes
the current view so that the camera position
matches the spotlight source location and the
camera target matches the light target point
(center of base of the cone). The camera angle
matches the light cone angle. This can verify a
light’s view as an analysis aid in setting up a scene.
‘PointLight’
Inserts a light object that emits light from a single
location in all directions (omni light).
Where is this tool:
Access this tool from Render>Set View to
Spotlight
Where is this tool:
Access this tool from Render>
Create Point Light
Steps:
Steps:
• Select Create Point Light from the Render menu.
• Pick the light’s location.
• Select a spot light in the viewport.
• Select Set Spotlight to View from the Render
menu.
## Create Directional Light
## Create Rectangular Light
‘DirectionalLight’
Inserts a light that simulates the sun defined with
parallel rays and a direction.
Where is this tool:
Access this tool from Render>Create
Directional Light
Steps:
‘RectangularLight’
Inserts a rectangular light object that emits light
from an array of light points in one direction.
Location point (1), length control (2), width control
(3), dolly point (4), target point (5).
Where is this tool:
Access this tool from Render>Create
Rectangular Light
Steps:
• Select Create Directional Light from the Render
menu.
• Pick a location at which to aim the directional light.
• Pick a location for the directional light.
• Select Create Rectangular Light from the Render
menu.
• Pick the light’s start point.
• Pick a the light’s length.
• Pick the light’s width.
Notes
• A directional light shines toward a
specified direction. The location of
the light does not matter: the light


---

## Images

- ![m8book2_p313_img01.png](../images/m8book2_p313_img01.png) page 313 (shared page)
- ![m8book2_p314_img01.png](../images/m8book2_p314_img01.png) page 314 (shared page)
- ![m8book2_p314_img02.png](../images/m8book2_p314_img02.png) page 314 (shared page)
- ![m8book2_p314_img03.png](../images/m8book2_p314_img03.png) page 314 (shared page)
