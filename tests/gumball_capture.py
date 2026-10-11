"""Capture the real viewport using the rendering API of the selected host."""
from PySide6 import QtCore


def capture_view(view, width, height):
    viewer = view.getViewer()
    if hasattr(viewer, 'renderToImage'):
        image = viewer.renderToImage(width=width, height=height, samples=0)
    else:
        # Stock FreeCAD 1.1 exposes its actual OpenGL framebuffer as a QImage.
        image = viewer.grabFramebuffer()
        if not image.isNull():
            image = image.scaled(width, height, QtCore.Qt.IgnoreAspectRatio,
                                 QtCore.Qt.SmoothTransformation)
    if image.isNull():
        raise RuntimeError('The native viewport produced no image')
    return image
