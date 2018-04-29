The `dmesktop` program is in many ways like `dmenu_run`, but it draws the list of programs to select from using the [XDG Desktop Entry specification](https://specifications.freedesktop.org/desktop-entry-spec/desktop-entry-spec-latest.html) instead of just using applications on the `$PATH` (which includes non-graphical programs you might not need to start via such a menu, like `ls` or `sed`).

The `dmesktop` script requires Python 3.6 or greater, and hasn't been tested on any machine but my own.

# Known Alternatives

- [i3-dmenu-desktop](https://build.i3wm.org/docs/i3-dmenu-desktop.html) is the same basic program, but it assumes that you're using `i3` to start the programs in question.
- [rofi](https://github.com/DaveDavenport/rofi) is a much more full-featured `dmenu`-like application switcher/launcher which includes this as a mode, in addition to many other modes
