# Debug Commands for Hyprshell

This document lists all available debug commands in Hyprshell CLI, along with sample usage for each command.

## Debug Commands

Debug commands are available when Hyprshell is built with the `debug_command` feature. These commands help with debugging various aspects of Hyprshell.

### List Icons

Lists all icons available in the current icon theme.

```bash
hyprshell debug list-icons
```

### List Desktop Files

Lists all desktop files found in the system.

```bash
hyprshell debug list-desktop-files
```

### Check Class

Searches for an icon associated with a specific window class. If no class is provided, all open windows will be searched.

```bash
# Check a specific class
hyprshell debug check-class "firefox"
```

```bash
# Check all open windows
hyprshell debug check-class
```

### Search

Simulates a search in the launcher and displays search insights. This helps debug the search functionality.

```bash
# Basic search
hyprshell debug search "terminal"
```

```bash
# Show all matches (not limited by config)
hyprshell debug search "terminal" --all
```

### Default Applications

Commands to manage default applications for different mime types.

#### Get Default App

Get the default application for a specific mime type.

```bash
hyprshell debug default-applications get "text/plain"
```

#### Add Default App

Add a default application for a specific mime type. If one already exists, the new one is placed before.

```bash
hyprshell debug default-applications add "text/plain" "org.gnome.gedit.desktop"
```

#### List Default Apps

List default applications for all mime types.

```bash
# List default apps for mime types used by Hyprshell (browser: x-scheme-handler/https, file manager: inode/director)
hyprshell debug default-applications list
```

```bash
# List default apps for all mime types
hyprshell debug default-applications list --all
```

#### Check Default Apps

Check if all entries in all mimetype files point to valid desktop files.

```bash
hyprshell debug default-applications check
```

## Data Commands

Data commands allow you to view and manage data stored by Hyprshell.

### Launch History

Shows the history of launched applications.

```bash
# Show launch history with default weeks setting from config
hyprshell data launch-history

# Show launch history for a specific number of weeks
hyprshell data launch-history 4
```

## Global Options

These options can be used with any command:

- `-v, -vv`: Increase verbosity level (-v: debug, -vv: trace)
- `-q, --quiet`: Turn off all output
- `-c, --config-file <PATH>`: Specify a custom config file path
- `-s, --css-file <PATH>`: Specify a custom CSS file path
- `-d, --data-dir <PATH>`: Specify a custom data directory path

Example with global options:

```bash
hyprshell -v -c ~/custom-config.ron debug list-icons
```