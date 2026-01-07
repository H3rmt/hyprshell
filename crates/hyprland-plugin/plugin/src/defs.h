#pragma once

#include <hyprland/src/plugins/PluginAPI.hpp>

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
#define HYPRSHELL_PRINT_DEBUG_DEBUG 0
#define HYPRSHELL_SOCKET_PATH "$HYPRSHELL_SOCKET_PATH$"

#define HYPRSHELL_MOD_ALT (1u << 0)
#define HYPRSHELL_MOD_CTRL (1u << 1)
#define HYPRSHELL_MOD_SUPER (1u << 2)
#define HYPRSHELL_MOD_SHIFT (1u << 3)

#define HYPRSHELL_HOLD_ALT (1u << 0)
#define HYPRSHELL_HOLD_CTRL (1u << 1)
#define HYPRSHELL_HOLD_SUPER (1u << 2)

$HYPRSHELL_SWITCH_BINDS$

#define HYPRSHELL_OVERVIEW_MOD "$HYPRSHELL_OVERVIEW_MOD$"
#define HYPRSHELL_OVERVIEW_KEY "$HYPRSHELL_OVERVIEW_KEY$"

#define HYPRSHELL_CLOSE R"($HYPRSHELL_CLOSE$)"
#define HYPRSHELL_OPEN_OVERVIEW R"($HYPRSHELL_OPEN_OVERVIEW$)"
// gets removed in the build process
#include "defs-test.h"
