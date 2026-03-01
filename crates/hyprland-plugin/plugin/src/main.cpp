#include <hyprland/src/plugins/PluginAPI.hpp>
#include "init.h"
#include "socket_listener.h"

// Do NOT change this function.
APICALL EXPORT std::string PLUGIN_API_VERSION() {
    return HYPRLAND_API_VERSION;
}

APICALL EXPORT PLUGIN_DESCRIPTION_INFO PLUGIN_INIT(HANDLE handle) {
    auto [name, description, author, version] = init(handle);
    return {name, description, author, version};
}

APICALL EXPORT void PLUGIN_EXIT() {
    HyprlandAPI::addNotification(PHANDLE, "[Hyprshell Plugin] Plugin deactivated", RED, 3000);
    stopSocketListener();
}
