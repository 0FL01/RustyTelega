# dmgbuild settings for Quill's disk image (scripts/macos-dmg.sh passes the
# paths with -D). The window is the background's size; icon positions are the
# centers the background draws its ribbon and arrow around (background.html).
import os.path

app = defines["app"]  # noqa: F821 (dmgbuild injects `defines`)
app_name = os.path.basename(app)

format = "ULMO"
filesystem = "APFS"
files = [app]
symlinks = {"Applications": "/Applications"}
icon = defines["volume_icon"]  # noqa: F821
background = defines["background"]  # noqa: F821

show_status_bar = False
show_tab_view = False
show_toolbar = False
show_pathbar = False
show_sidebar = False
window_rect = ((200, 120), (660, 400))
default_view = "icon-view"
show_icon_preview = False
arrange_by = None
icon_size = 128
text_size = 13
label_pos = "bottom"
icon_locations = {app_name: (170, 190), "Applications": (490, 190)}
hide_extensions = [app_name]
