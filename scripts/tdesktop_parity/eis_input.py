#!/usr/bin/python3 -I
"""Drive pointer and keyboard input in a nested KWin through its EIS server.

capture_reference.sh runs Telegram Desktop in a nested, headless KWin and
has to click and type in it. Nothing there accepts X11 or wlroots virtual
input, but KWin hands out libei sockets on its session bus
(org.kde.KWin.EIS.RemoteDesktop.connectToEIS), so this talks libei through
ctypes (libei.so.1; no headers needed) to the nested compositor only. It
never touches the host seat.

Commands, one per line on stdin or as arguments:
  move X Y        pointer to logical (X, Y)
  click X Y       move there and click the left button
  key NAME        press and release a key (enter, esc, tab, backspace, ...)
  type TEXT       type ASCII text on a US layout
  sleep MS        wait
  shot PATH       capture the active window with Spectacle
  maximize        maximize the active window (KWin script)

Run it with the system python (python3-dbus), inside the nested session.
"""
import ctypes
import os
import select
import subprocess
import sys
import tempfile
import time

import dbus

lib = ctypes.CDLL("libei.so.1")
P = ctypes.c_void_p
for name, res, args in [
    ("ei_new_sender", P, [P]),
    ("ei_configure_name", None, [P, ctypes.c_char_p]),
    ("ei_setup_backend_fd", ctypes.c_int, [P, ctypes.c_int]),
    ("ei_get_fd", ctypes.c_int, [P]),
    ("ei_dispatch", None, [P]),
    ("ei_get_event", P, [P]),
    ("ei_event_get_type", ctypes.c_int, [P]),
    ("ei_event_get_seat", P, [P]),
    ("ei_event_get_device", P, [P]),
    ("ei_event_unref", P, [P]),
    ("ei_device_has_capability", ctypes.c_bool, [P, ctypes.c_int]),
    ("ei_device_start_emulating", None, [P, ctypes.c_uint32]),
    ("ei_device_pointer_motion_absolute", None, [P, ctypes.c_double, ctypes.c_double]),
    ("ei_device_button_button", None, [P, ctypes.c_uint32, ctypes.c_bool]),
    ("ei_device_keyboard_key", None, [P, ctypes.c_uint32, ctypes.c_bool]),
    ("ei_device_frame", None, [P, ctypes.c_uint64]),
    ("ei_device_ref", P, [P]),
    ("ei_now", ctypes.c_uint64, [P]),
]:
    fn = getattr(lib, name)
    fn.restype = res
    fn.argtypes = args
lib.ei_seat_bind_capabilities.restype = None

EV_CONNECT, EV_DISCONNECT, EV_SEAT_ADDED = 1, 2, 3
EV_DEVICE_ADDED, EV_DEVICE_RESUMED = 5, 8
CAP_POINTER_ABS, CAP_KEYBOARD, CAP_BUTTON = 1 << 1, 1 << 2, 1 << 5
BTN_LEFT = 0x110

KEYS = {
    "esc": 1, "backspace": 14, "tab": 15, "enter": 28, "space": 57,
    "left": 105, "right": 106, "up": 103, "down": 108, "home": 102,
    "end": 107, "delete": 111, "ctrl": 29, "shift": 42, "alt": 56,
}
ROWS = [
    ("1234567890-=", 2), ("qwertyuiop[]", 16), ("asdfghjkl;'", 30),
    ("zxcvbnm,./", 44),
]
SHIFTED = dict(zip('!@#$%^&*()_+{}:"<>?', '1234567890-=[];\',./'))
CHARS = {" ": 57, "`": 41, "\\": 43}
for row, first in ROWS:
    for i, ch in enumerate(row):
        CHARS[ch] = first + i


class Eis:
    def __init__(self):
        bus = dbus.SessionBus()
        obj = bus.get_object("org.kde.KWin", "/org/kde/KWin/EIS/RemoteDesktop")
        fd, self.cookie = obj.connectToEIS(
            dbus.Int32(63), dbus_interface="org.kde.KWin.EIS.RemoteDesktop")
        self.ei = lib.ei_new_sender(None)
        lib.ei_configure_name(self.ei, b"quill-parity")
        if lib.ei_setup_backend_fd(self.ei, os.dup(fd.take())) != 0:
            raise SystemExit("ei_setup_backend_fd failed")
        self.pointer = None
        self.keyboard = None
        self.seq = 1
        deadline = time.time() + 5
        while (self.pointer is None or self.keyboard is None) and time.time() < deadline:
            self.pump(0.2)
        if self.pointer is None or self.keyboard is None:
            raise SystemExit("no EIS pointer/keyboard device")

    def pump(self, timeout):
        select.select([lib.ei_get_fd(self.ei)], [], [], timeout)
        lib.ei_dispatch(self.ei)
        while True:
            ev = lib.ei_get_event(self.ei)
            if not ev:
                break
            kind = lib.ei_event_get_type(ev)
            if kind == EV_SEAT_ADDED:
                seat = lib.ei_event_get_seat(ev)
                lib.ei_seat_bind_capabilities(
                    P(seat), ctypes.c_int(CAP_POINTER_ABS), ctypes.c_int(CAP_BUTTON),
                    ctypes.c_int(CAP_KEYBOARD), P(None))
            elif kind == EV_DEVICE_RESUMED:
                dev = lib.ei_event_get_device(ev)
                lib.ei_device_ref(dev)
                lib.ei_device_start_emulating(dev, self.seq)
                self.seq += 1
                if lib.ei_device_has_capability(dev, CAP_POINTER_ABS):
                    self.pointer = dev
                if lib.ei_device_has_capability(dev, CAP_KEYBOARD):
                    self.keyboard = dev
            elif kind == EV_DISCONNECT:
                raise SystemExit("EIS disconnected")
            lib.ei_event_unref(ev)

    def frame(self, dev):
        lib.ei_device_frame(dev, lib.ei_now(self.ei))
        lib.ei_dispatch(self.ei)
        self.pump(0.02)

    def move(self, x, y):
        lib.ei_device_pointer_motion_absolute(self.pointer, float(x), float(y))
        self.frame(self.pointer)

    def click(self, x, y):
        self.move(x, y)
        time.sleep(0.05)
        for down in (True, False):
            lib.ei_device_button_button(self.pointer, BTN_LEFT, down)
            self.frame(self.pointer)
            time.sleep(0.05)

    def key(self, code, shift=False):
        dev = self.keyboard
        if shift:
            lib.ei_device_keyboard_key(dev, KEYS["shift"], True)
            self.frame(dev)
        for down in (True, False):
            lib.ei_device_keyboard_key(dev, code, down)
            self.frame(dev)
            time.sleep(0.02)
        if shift:
            lib.ei_device_keyboard_key(dev, KEYS["shift"], False)
            self.frame(dev)

    def type(self, text):
        for ch in text:
            if ch in SHIFTED:
                self.key(CHARS[SHIFTED[ch]], True)
            elif ch.isupper():
                self.key(CHARS[ch.lower()], True)
            else:
                self.key(CHARS[ch])
            time.sleep(0.03)


def maximize():
    with tempfile.NamedTemporaryFile("w", suffix=".js", delete=False) as js:
        js.write("const w = workspace.activeWindow;"
                 " if (w) { w.setMaximize(true, true); }")
    bus = dbus.SessionBus()
    scripting = bus.get_object("org.kde.KWin", "/Scripting")
    sid = scripting.loadScript(js.name, "quill-maximize",
                               dbus_interface="org.kde.kwin.Scripting",
                               signature="ss")
    script = bus.get_object("org.kde.KWin", f"/Scripting/Script{sid}")
    script.run(dbus_interface="org.kde.kwin.Script")
    time.sleep(0.5)
    scripting.unloadScript("quill-maximize", dbus_interface="org.kde.kwin.Scripting")
    os.unlink(js.name)


def run(eis, line):
    parts = line.strip().split(" ", 1)
    if not parts[0] or parts[0].startswith("#"):
        return
    cmd, rest = parts[0], (parts[1] if len(parts) > 1 else "")
    if cmd == "move":
        eis.move(*map(float, rest.split()))
    elif cmd == "click":
        eis.click(*map(float, rest.split()))
    elif cmd == "key":
        eis.key(KEYS[rest.strip()])
    elif cmd == "type":
        eis.type(rest)
    elif cmd == "shot":
        subprocess.run(["spectacle", "--background", "--nonotify",
                        "--activewindow", "--output", rest.strip()],
                       check=True, stdout=subprocess.DEVNULL,
                       stderr=subprocess.DEVNULL)
    elif cmd == "maximize":
        maximize()
    elif cmd == "sleep":
        end = time.time() + float(rest) / 1000
        while time.time() < end:
            eis.pump(min(0.1, max(0.0, end - time.time())))
    else:
        raise SystemExit(f"unknown command: {line}")


def main():
    eis = Eis()
    lines = sys.argv[1:] or sys.stdin
    for line in lines:
        run(eis, line)


if __name__ == "__main__":
    main()
