"""Capture the real viewport using the rendering API of the selected host."""
from pathlib import Path
import tempfile
from PySide6 import QtGui


def capture_view(view, width, height):
    viewer = view.getViewer()
    if hasattr(viewer, 'renderToImage'):
        image = viewer.renderToImage(width=width, height=height, samples=0)
    else:
        # Stock 1.1 has no renderToImage. Its isolated CI profile selects the
        # native FramebufferObject SavePicture method in the viewer's context.
        # Render at the requested size instead of stretching a stale framebuffer.
        with tempfile.TemporaryDirectory(prefix='om9-native-capture-') as directory:
            destination = Path(directory) / 'viewport.png'
            view.saveImage(str(destination), width, height, 'Current')
            image = QtGui.QImage(str(destination))
    if image.isNull():
        raise RuntimeError('The native viewport produced no image')
    return image
