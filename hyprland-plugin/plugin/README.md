# Plugin build at runtime of Hyprshell

The Makefile includes all necessary steps to develop.

For performance reasons all .cpp files are combined into one file, `make prepare` will be run to do that before zipping the plugin and including it in the hyprshell binary.

`make` builds the plugin, it will be placed in `out/hyprshell.so`.