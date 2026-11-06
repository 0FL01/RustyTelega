"""Per-screen parity profiles for align.py and diff.py.

A profile names the reference size and, in that reference's pixels
(x0, y0, x1, y1), the regions diff.py reports, the text runs align.py checks,
the boxes --masks leaves out and the boxes that are always left out (the
reference's mouse pointer, content no fixture can reproduce).

"main" is the 1919x1034 main window (artifacts/telegram-donor.png). The
"intro-*" profiles are the 1920x1036 maximized Telegram Desktop 7.3.5 intro
screens that capture_reference.sh takes (scenario reference/intro.txt).
"""

MAIN = {
    "size": (1919, 1034),
    "regions": {
        "titlebar": (0, 0, 1919, 24),
        "folder-rail": (0, 24, 72, 1034),
        "chat-list": (72, 24, 525, 1034),
        "header": (526, 24, 1919, 79),
        "pinned": (526, 79, 1919, 128),
        "history": (526, 128, 1919, 990),
        "composer": (526, 990, 1919, 1034),
    },
    "runs": {
        "search placeholder": (96, 42, 160, 60),
        "rail label": (8, 118, 64, 132),
        "list name 1": (158, 90, 300, 106),
        "list date 1": (455, 90, 516, 106),
        "list preview 1": (160, 113, 515, 130),
        "list name 2": (158, 152, 250, 168),
        "list preview 2": (160, 175, 515, 192),
        "header name": (540, 34, 600, 50),
        "header status": (540, 55, 690, 70),
        "pinned title": (544, 86, 660, 101),
        "pinned text": (544, 104, 830, 119),
        "message line 1": (590, 254, 995, 270),
        "message line 2": (590, 272, 860, 288),
        "message time": (935, 422, 998, 436),
        "reply name": (636, 463, 720, 478),
        "reply link": (636, 481, 850, 496),
        "voice status": (646, 536, 745, 551),
        "voice time": (903, 541, 965, 556),
        "file name": (646, 689, 1003, 704),
        "file duration": (646, 711, 686, 726),
        "gif time": (828, 953, 892, 969),
        "composer placeholder": (574, 1005, 690, 1022),
    },
    # Fixture content that cannot match without the reference's pictures.
    "masks": [
        (82, 86, 128, 318),     # chat list avatars
        (536, 128, 576, 990),   # history avatars
        (580, 128, 902, 238),   # photo at the top of the history
        (580, 800, 902, 980),   # GIF
    ],
    "always": [(378, 702, 398, 730)],  # the mouse pointer in the reference
}

INTRO_SIZE = (1920, 1036)
TITLEBAR = (0, 0, 1920, 24)
# Settings and the back arrow, shown on every step after the cover.
SHELL_RUNS = {
    "back arrow": (16, 40, 42, 64),
    "settings": (1824, 41, 1898, 62),
}

INTRO_START = {
    "size": INTRO_SIZE,
    "regions": {
        "titlebar": TITLEBAR,
        "cover": (0, 24, 1920, 232),
        "text": (0, 232, 1920, 580),
        "button": (0, 580, 1920, 650),
        "rest": (0, 650, 1920, 1036),
    },
    "runs": {
        "cover title": (860, 467, 1058, 495),
        "cover text 1": (795, 504, 1123, 526),
        "cover text 2": (889, 528, 1028, 546),
        "button label": (900, 603, 1020, 626),
    },
    "masks": [],
    "always": [],
}

INTRO_QR = {
    "size": INTRO_SIZE,
    "regions": {
        "titlebar": TITLEBAR,
        "shell": (0, 24, 1920, 80),
        "code": (860, 310, 1060, 510),
        "text": (780, 510, 1140, 660),
        "links": (780, 660, 1140, 760),
    },
    "runs": dict(SHELL_RUNS, **{
        "qr title": (825, 525, 1093, 552),
        "qr step 1": (816, 567, 1022, 587),
        "qr step 2": (816, 593, 1063, 613),
        "qr step 3": (816, 619, 987, 639),
        "phone link": (874, 687, 1046, 707),
        "passkey link": (896, 714, 1024, 734),
    }),
    # Integer-scaled 41-module matrix; preserve the static disc and plane.
    "masks": [(878, 329, 1041, 492)],
    "keep_ellipses": [(938, 389, 981, 432)],
    "always": [],
}

PHONE_RUNS = dict(SHELL_RUNS, **{
    "phone title": (811, 330, 984, 349),
    "phone text 1": (812, 362, 1020, 382),
    "phone text 2": (811, 382, 1004, 402),
    "country name": (810, 451, 876, 471),
    "country arrow": (1086, 453, 1106, 472),
    "country code": (824, 518, 862, 537),
    "phone digits": (890, 517, 1000, 538),
    "button label": (936, 603, 984, 626),
    "qr link": (876, 695, 1042, 715),
})

INTRO_PHONE = {
    "size": INTRO_SIZE,
    "regions": {
        "titlebar": TITLEBAR,
        "shell": (0, 24, 1920, 80),
        "text": (780, 320, 1140, 410),
        "country": (780, 440, 1140, 490),
        "fields": (780, 500, 1140, 556),
        "error": (780, 556, 1140, 590),
        "button": (780, 590, 1140, 640),
        "link": (780, 680, 1140, 730),
    },
    "runs": PHONE_RUNS,
    "masks": [],
    "always": [],
}

INTRO_PHONE_ERROR = dict(INTRO_PHONE, runs=dict(PHONE_RUNS, **{
    "error": (837, 563, 1083, 583),
}))

INTRO_SETTINGS = {
    "size": INTRO_SIZE,
    "regions": {
        "titlebar": TITLEBAR,
        "box": (764, 64, 1156, 802),
        "backdrop": (0, 24, 764, 1036),
    },
    "runs": {
        "settings title": (786, 82, 860, 104),
        "language": (822, 143, 894, 163),
        "language value": (1084, 143, 1135, 163),
        "connection type": (822, 183, 934, 203),
        "connection value": (1011, 183, 1136, 203),
        "automatic updates": (784, 241, 932, 262),
        "beta versions": (784, 300, 922, 321),
        "window frame": (821, 397, 970, 418),
        "interface scale": (784, 458, 937, 479),
        "scale value": (1099, 491, 1136, 511),
        "theme classic": (803, 628, 850, 648),
        "theme day": (900, 628, 932, 648),
        "theme tinted": (983, 628, 1026, 648),
        "theme night": (1074, 628, 1114, 648),
        "system accent": (821, 700, 961, 721),
        "faq": (784, 768, 880, 789),
    },
    # Only the version and update status vary between builds/runs.
    "masks": [(784, 264, 1096, 282), (784, 337, 1136, 360)],
    "always": [],
}

PROFILES = {
    "main": MAIN,
    "intro-start": INTRO_START,
    "intro-qr": INTRO_QR,
    "intro-phone": INTRO_PHONE,
    "intro-phone-filled": INTRO_PHONE,
    "intro-phone-error": INTRO_PHONE_ERROR,
    "intro-settings": INTRO_SETTINGS,
}


def take(args):
    """Pop "--profile NAME" from args; return (profile, remaining args)."""
    name = "main"
    rest = []
    it = iter(args)
    for a in it:
        if a == "--profile":
            name = next(it)
        elif a.startswith("--profile="):
            name = a.split("=", 1)[1]
        else:
            rest.append(a)
    if name not in PROFILES:
        raise SystemExit(f"unknown profile {name!r}; known: {', '.join(PROFILES)}")
    return PROFILES[name], rest


def check_size(profile, img, what):
    if img.size != tuple(profile["size"]):
        raise SystemExit(
            f"{what} is {img.size[0]}x{img.size[1]}, the profile expects "
            f"{profile['size'][0]}x{profile['size'][1]}")
