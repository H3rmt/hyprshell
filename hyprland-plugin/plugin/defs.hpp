#pragma once

#include <hyprland/src/plugins/PluginAPI.hpp>

// gets removed in the build process
#include "defs-test.hpp"

const CHyprColor RED{1.0, 0.2, 0.2, 1.0};
const CHyprColor GREEN{0.2, 1.0, 0.2, 1.0};

struct PluginDescriptionInfo {
    std::string name;
    std::string description;
    std::string author;
    std::string version;
};

#define HYPRSHELL_PLUGIN_NAME "$HYPRSHELL_PLUGIN_NAME$"
#define HYPRSHELL_PLUGIN_AUTHOR "$HYPRSHELL_PLUGIN_AUTHOR$"
#define HYPRSHELL_PLUGIN_DESC "$HYPRSHELL_PLUGIN_DESC$"
#define HYPRSHELL_PLUGIN_VERSION "$HYPRSHELL_PLUGIN_VERSION$"

#define HYPRSHELL_PRINT_DEBUG $HYPRSHELL_PRINT_DEBUG$

#define HYPRSHELL_SWTICH_XKB_KEY_L $HYPRSHELL_SWTICH_XKB_MOD_L$
#define HYPRSHELL_SWTICH_XKB_KEY_R $HYPRSHELL_SWTICH_XKB_MOD_R$
#define HYPRSHELL_PROGRAM_CLOSE_SWITCH R"($HYPRSHELL_PROGRAM_CLOSE_SWITCH$)"
