#include <hyprland/src/plugins/PluginAPI.hpp>

#include "globals.hpp"

// Do NOT change this function.
APICALL EXPORT std::string PLUGIN_API_VERSION() {
    return HYPRLAND_API_VERSION;
}


APICALL EXPORT PLUGIN_DESCRIPTION_INFO PLUGIN_INIT(HANDLE handle) {
    PHANDLE = handle;
    const std::string name = HYPRSHELL_PLUGIN_NAME;
    const std::string author = HYPRSHELL_PLUGIN_AUTHOR;
    const std::string description = HYPRSHELL_PLUGIN_DESC;
    const std::string version = HYPRSHELL_PLUGIN_VERSION;
    return {name, description, author, version};
}

APICALL EXPORT void PLUGIN_EXIT() {
}
