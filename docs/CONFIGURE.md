# Config

The main config file is located at `~/.config/hyprshell/config.toml`. 

Support for the `.ron` and `.json(5)` file formats was removed so the config can use `toml_edit`, which preserves comments and other user changes.
The config is loaded at startup and is reloaded when the file changes.

To interactively generate a default config file with all possible options set, run the following command:

```bash
hyprshell config generate
```

In case this documentation is outdated, or you understand rust, look at the [struct definition](../crates/config-lib/src/io/structs.rs) for the most up-to-date information.

The default values for these configs, which are also the values that get used when generating the config, are located in the code directly above the value definition (`#[default ... ]`).

## Config Options

Explanation was moved from this Document to the GUI Settings Editor (start with `hyprshell config edit`).

# CSS

The CSS file is located at `~/.config/hyprshell/styles.css` but can be configured using the `-s` argument.
The config is loaded at startup and is reloaded when the file changes. (removing styles will not work, adding or overriding styles works)


> [!CAUTION]
> This is out of date, use the config editor instead, it contains premade styles, including a testing theme which applies different colors to all elements.
> Themes are located under /usr/share/hyprshell/themes/ after install or [in this repo](../packaging/share/themes)

GTK only supports a subset of CSS, so not all CSS properties will work. The supported properties are listed in the [GTK documentation](https://docs.gtk.org/gtk4/css-overview.html).

These settings will take priority over the default values set by the application itself.
The application defaults can be found in the CSS files inside the codebase (for example, [this one](../src/fallback-styles.css) or [that one](../crates/windows-lib/src/styles.css)).

If you want to change colors borders, etc. you can edit the CSS variables in the `:root {}` section.
These styles are automatically used everywhere in the application, so you don't have to set them for every class.
