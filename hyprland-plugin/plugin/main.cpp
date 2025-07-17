#include <hyprland/src/plugins/PluginAPI.hpp>
#include <hyprland/src/includes.hpp>

#include "globals.hpp"
#include "defs.hpp"

// Do NOT change this function.
APICALL EXPORT std::string PLUGIN_API_VERSION() {
    return HYPRLAND_API_VERSION;
}

void onOpenLayerChange(const PHLLS &window) {
    HyprlandAPI::addNotification(PHANDLE, "AMONGUS", CHyprColor{1.0, 0.2, 0.2, 1.0}, 5000);
}

APICALL EXPORT PLUGIN_DESCRIPTION_INFO PLUGIN_INIT(HANDLE handle) {
    PHANDLE = handle;
    HyprlandAPI::addNotification(PHANDLE, "---AMONGUS----", CHyprColor{1.0, 0.2, 0.2, 1.0}, 5000);

    // ALWAYS add this to your plugins. It will prevent random crashes coming from
    // mismatched header versions.
    if (const std::string HASH = __hyprland_api_get_hash(); HASH != GIT_COMMIT_HASH) {
        HyprlandAPI::addNotification(PHANDLE, "[MyPlugin] Mismatched headers! Can't proceed.",
                                     CHyprColor{1.0, 0.2, 0.2, 1.0}, 5000);
        throw std::runtime_error("[MyPlugin] Version mismatch");
    }

    // clang-format off
    static auto P = HyprlandAPI::registerCallbackDynamic(PHANDLE, "openLayer",[&](void* self, SCallbackInfo& info, std::any data) { onOpenLayerChange(std::any_cast<PHLLS>(data)); });
    // clang-format on

    const std::string name = HYPRSHELL_PLUGIN_NAME;
    const std::string author = HYPRSHELL_PLUGIN_AUTHOR;
    const std::string description = HYPRSHELL_PLUGIN_DESC;
    const std::string version = HYPRSHELL_PLUGIN_VERSION;
    return {name, description, author, version};
}

APICALL EXPORT void PLUGIN_EXIT() {
}
