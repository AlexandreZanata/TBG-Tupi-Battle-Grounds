#!/usr/bin/env python3
"""Drive real X11 mouse/keyboard input on $DISPLAY for headless MOBA checks.

Usage (inside `xvfb-run`): xtest_input.py click:X:Y[:BUTTON] move:X:Y key:NAME
focus sleep:SECONDS ...   `focus` gives the window under the pointer the
keyboard (Xvfb has no window manager, so keys are lost without it).
Needs only libX11 and libXtst (no xdotool).
"""
import ctypes
import sys
import time

x11 = ctypes.CDLL("libX11.so.6")
xt = ctypes.CDLL("libXtst.so.6")
x11.XOpenDisplay.restype = ctypes.c_void_p
x11.XStringToKeysym.restype = ctypes.c_ulong
x11.XDefaultRootWindow.restype = ctypes.c_ulong
x11.XDefaultRootWindow.argtypes = [ctypes.c_void_p]
display = ctypes.c_void_p(x11.XOpenDisplay(None))


def flush():
    x11.XFlush(display)


def move(x, y):
    xt.XTestFakeMotionEvent(display, -1, int(x), int(y), 0)
    flush()
    time.sleep(0.08)


def click(x, y, button=1):
    move(x, y)
    for down in (1, 0):
        xt.XTestFakeButtonEvent(display, button, down, 0)
        flush()
        time.sleep(0.08)


def key(name):
    code = x11.XKeysymToKeycode(display, x11.XStringToKeysym(name.encode()))
    for down in (1, 0):
        xt.XTestFakeKeyEvent(display, code, down, 0)
        flush()
        time.sleep(0.08)


def focus():
    root = ctypes.c_ulong(x11.XDefaultRootWindow(display))
    r, child, i, m = ctypes.c_ulong(), ctypes.c_ulong(), ctypes.c_int(), ctypes.c_uint()
    x11.XQueryPointer(display, root, ctypes.byref(r), ctypes.byref(child),
                      ctypes.byref(i), ctypes.byref(i), ctypes.byref(i), ctypes.byref(i), ctypes.byref(m))
    x11.XSetInputFocus(display, child, 2, 0)
    flush()
    time.sleep(0.3)


for command in sys.argv[1:]:
    op, *args = command.split(":")
    if op == "click":
        click(float(args[0]), float(args[1]), int(args[2]) if len(args) > 2 else 1)
    elif op == "move":
        move(float(args[0]), float(args[1]))
    elif op == "key":
        key(args[0])
    elif op == "focus":
        focus()
    elif op == "sleep":
        time.sleep(float(args[0]))
    else:
        sys.exit(f"unknown step {command!r}")
