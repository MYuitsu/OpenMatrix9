"""Native fixture helpers for the one-document Command surface.

Replace only the live input suffix using QTextCursor; key events still target
the actual visible console. This is fixture code, not a product input proxy.
"""
from PySide6 import QtGui, QtWidgets


def command_input(window):
    field = window.findChild(QtWidgets.QPlainTextEdit, 'OM9CommandTranscript')
    if field is None:
        return None

    def text():
        cursor = QtGui.QTextCursor(field.document())
        cursor.setPosition(int(field.property('om9InputStart')))
        cursor.movePosition(QtGui.QTextCursor.End, QtGui.QTextCursor.KeepAnchor)
        return cursor.selectedText()

    def set_text(value):
        cursor = QtGui.QTextCursor(field.document())
        cursor.setPosition(int(field.property('om9InputStart')))
        cursor.movePosition(QtGui.QTextCursor.End, QtGui.QTextCursor.KeepAnchor)
        cursor.insertText(value)
        field.setTextCursor(cursor)

    field.text = text
    field.setText = set_text
    field.clear = lambda: set_text('')
    field.selectedText = lambda: field.textCursor().selectedText()
    return field


def command_prompt(window):
    class Prompt:
        def text(self):
            field = window.findChild(QtWidgets.QPlainTextEdit, 'OM9CommandTranscript')
            return field.property('om9Prompt') if field else ''

    return Prompt()
