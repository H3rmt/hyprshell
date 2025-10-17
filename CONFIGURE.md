# Config

The main config file is located at `~/.config/hyprshell/config.ron` but can be configured using the `-c` argument. You can also use `.json` and `.json5` as config file formats.
The config is loaded at startup but is reloaded when the file changes.

To generate a default config file with all possible options set, run the following command:

```bash
hyprshell config generate
```

In case this documentation is outdated, or you understand rust, look at the [struct definition](crates/config-lib/src/structs.rs) for the most up-to-date information.

The default values for these configs, which are also the values that get used when generating the config, are located in the code directly above the value definition (`#[default ... ]`).

## Config Options

- **version:**_[number]_ The version of the config file. Will be used for the migrator in the future.
- **windows:**_[Windows]_ Configuration for the different windows like overview, switch and launcher (optional).

## Windows

- **scale:**_[number]_ The scale used to scale down the real dimension the windows displayed in the overview. Can be set from `0.0 < X > to 15.0`
- **items_per_row:**_[number]_ The number of workspaces or windows to show per row in the overview. If you have 6 workspaces open and set this to 3, you will see 2 rows of 3 workspaces.
  Pressing arrow up or down switches between the rows.
- **overview:**_[Overview]_ Configuration for the overview mode (optional).
- **switch:**_[Switch]_ Configuration for the switch mode (optional).

### Overview

This mode displays the windows in a downscaled view of the screen. It also shows the launcher. This option itself is optional, if not set, this mode is disabled.

- **launcher:**_[Launcher]_ Configuration for the launcher.
- **key:**_[string]_ The key to use to open the Overview mode (like "tab" or "alt_r"). This is used to register the keybinding to open the Overview mode. If you want to only open using a modifier, set this to the modifier name like `super_l`.
- **modifier:**_[string]_ The modifier that must be pressed together with the key to open the Overview mode (like ctrl). This MUST be one of these modifiers: `alt, ctrl, super`.
- **filter_by:**_[List<FilterBy>]_ Filter the windows by the provided filter. This is a list of the following objects. (example: `filter_by: [current_workspace]`)
    - **same_class**: Only includes windows of the same class / type. If you currently have alacritty open, only alacritty windows will be shown.
    - **current_workspace**: Only includes windows of the current workspace.
    - **current_monitor**: Only includes windows of the current monitor.
- **hide_filtered:**_[boolean]_ whether to hide the filtered windows or not. This is used to show the windows that are filtered out by the `filter_by` option. If this is set to false, the filtered windows are shown with a grayscale effect.

## Launcher

- **default_terminal:**_[string]_ Defined the name of the default terminal to use. This value is optional, if unset a list of [default terminals](crates/core-lib/src/util/const.rs) is used to find a default terminal.
  This is used to launch programs like micro from the launcher that need to be run in a terminal.
  This terminal is also used by the `terminal` plugin to run the typed command in a terminal.
- **launch_modifier:**_[string]_ Sets the modifier used to launch apps in the launcher by pressing `<Mod> + 1` to open second, `<Mod> + t` to run in terminal, etc. This MUST be one of these modifiers: `alt, ctrl, super`.
- **width:**_[number]_ The width of the launcher in pixels.
- **max_items:**_[number]_ Sets the maximum number of items to show in the launcher.
  This does not include the plugin row and only limits the number of items retuned by for examples the application search.
  This value will get reduced to 10 if it is set to a value higher than 10.
- **show_when_empty:**_[boolean]_ Show entries in the launcher when no text is entered. This will show applications sorted by usage
- **plugins:**_[Plugins]_ Configuration for each Plugin. Ignore the individual plugins to disable them.

### Plugins

- **applications:** Show installed applications in the launcher, filed by the input, sorted by how often they are used. The following options can be provided:
    - **run_cache_weeks:**_[u8]_ How many weeks to cache the run history. This is used to sort the applications by how often they are used.
    - **show_execs:**_[boolean]_ Show the exec line from the Desktop file. In the case of Flatpaks and PWAs these get shortened to the name of the app.
      The full exec can still be seen in the tooltip.
    - **show_actions_submenu:**_[boolean]_ Show a dropdown menu with all the desktop actions specified in the `.desktop` files of the applications, like `new private window`, etc.
- **terminal:** Open a terminal and run the typed command in it. The terminal is defined in the `default_terminal` config option. This plugin doesn't accept any options.
- **shell:** Run the typed command in a shell (in the background). This plugin doesn't accept any options.
- **webSearch:** Allows searching for the typed query in a web browser.
    - **engines:**_[List<WebSearchEngine>]_ A list of search engines to use. Each search engine is defined by the following properties.
        - **url:**_[string]_ URL to open in the browser. This must include a `{}` to replace with the searched text (like `https://www.google.com/search?q={}`).
        - **name:**_[string]_ Name of the search engine. This is used to show the name in the launcher.
        - **key:**_[string]_ Key to use to select this search engine. This is used to register the keybinding to select the search engine without clicking on it.
- **calc:** Calculates any mathematical expression typed into the launcher. This plugin doesn't accept any options.
- **path:** Opens the selected path in the default file manager (see [Debugging](./DEBUG.md) to check default). This plugin doesn't accept any options.
- **actions:** Runs the specified action like reboot, hibernate, etc. Custom actions can also be specified.
    - **actions:**_[List<Action>]_ A list of actions to display in the launcher. Actions can be one of the following predefined actions or a custom action.
        - **lock_screen** Locks the screen.
        - **hibernate** Hibernates the system (copys the RAM to disk and powers off).
        - **logout** Logs out the user.
        - **reboot** Reboots the system.
        - **shutdown** Shuts down the system.
        - **suspend** Suspends the system.
        - **custom** A list of custom actions to run. Each action is defined by the following properties.
            - **names:**_[List<string>]_ List of names to use for the action, like `["poweroff", "shutdown"]`.
            - **details:**_[string]_ Details about the action. This is used to show the details in the launcher.
            - **command:**_[string]_ Command to run when the action is selected. (example: `command: "sudo shutdown -h now"`).
              can include `{}` which is replaced with the content of the text in the launcher (without the name of the action).
              Typing `kill 100` and running the action kill with a name `kill` would replace `{}` with `100`.
            - **icon:**_[string]_ Icon to show in the launcher. (you can find icons using the `hyprshell debug list-icons` command)

### Switch

This mode displays the windows sorted by their most recent access. This option itself is optional, if not set, this mode is disabled.

- **modifier:**_[string]_ The modifier that must be helled down together with `tab` key to open the Switch mode (for example `alt`). Letting go of this key will close the Switch mode. This MUST be one of these modifiers: `alt, ctrl, super`.
- **filter_by**_[List<FilterBy>]_ Filter the windows by the provided filter. This is a list of `FilterBy` objects. (example: `filter_by: [current_workspace]`)
    - **same_class:** Only includes windows of the same class / type. If you currently have alacritty open, only alacritty windows will be shown.
    - **current_workspace:** Only includes windows of the current workspace.
    - **current_monitor:** Only includes windows of the current monitor.
- **switch_workspaces:**_[boolean]_ Show the workspaces in the Switch mode instead of the windows.

# CSS

The CSS file is located at `~/.config/hyprshell/styles.css` but can be configured using the `-s` argument. The config is loaded at startup but is reloaded when the file changes. (removing styles will not work, adding or overriding styles works)

**Some examples can be found in the [CSS Examples folder](./css-examples).**

To generate a default file with all possible classes and CSS variables, run the following command:

```bash
hyprshell config generate
```

GTK only supports a subset of CSS, so not all CSS properties will work. The supported properties are listed in the [GTK documentation](https://docs.gtk.org/gtk4/css-overview.html).

The override file contains many empty classes that can be used to configure padding, fonts, etc.
These settings will take priority over the default values set by the application itself. The application defaults can be found in the CSS files inside the codebase (for example, [this one](src/default_styles.css) or [that one](crates/windows-lib/src/styles.css)).

If you want to change colors borders, etc. you can edit the CSS variables in the `:root {}` section.
These styles are automatically used everywhere in the application, so you don't have to set them for every class.
The values in the `:root {}` are set as fallbacks everywhere in the application, so you can just not set the ones you don't want to change.

![image.png](./imgs/css/swappy-20250510_222852.png)
![image.png](./imgs/css/swappy-20250510_224344.png)
