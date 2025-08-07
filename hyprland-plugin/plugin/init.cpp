#include "globals.hpp"
#include "handlers.hpp"
#include "defs.hpp"

PluginDescriptionInfo init(HANDLE handle) {
    PHANDLE = handle;

    if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
        HyprlandAPI::addNotification(PHANDLE, "[Hyprshell Plugin] Plugin started", GREEN, 3000);
    }

    // ALWAYS add this to your plugins. It will prevent random crashes coming from
    // mismatched header versions.
    if (const std::string HASH = __hyprland_api_get_hash(); HASH != GIT_COMMIT_HASH) {
        HyprlandAPI::addNotification(
            PHANDLE,
            "[Hyprshell Plugin] Mismatched headers! Can't proceed. (Hyprland was updated but not restarted)", RED,
            5000);
        throw std::runtime_error("[Hyprshell Plugin] Version mismatch");
    }

    // clang-format off
    static auto P1 = HyprlandAPI::registerCallbackDynamic(PHANDLE, "openLayer",[&](void*, SCallbackInfo&, const std::any &data) { onOpenLayerChange(std::any_cast<PHLLS>(data), true); });
    static auto P2 = HyprlandAPI::registerCallbackDynamic(PHANDLE, "closeLayer",[&](void*, SCallbackInfo&, const std::any &data) { onOpenLayerChange(std::any_cast<PHLLS>(data), false); });
    static auto P3 = HyprlandAPI::registerCallbackDynamic(PHANDLE, "keyPress",[&](void*, SCallbackInfo&, const std::any &data) { onKeyPress(std::any_cast<std::unordered_map<std::string, std::any>>(data)); });
    // clang-format on

    const std::string name = HYPRSHELL_PLUGIN_NAME;
    const std::string author = HYPRSHELL_PLUGIN_AUTHOR;
    const std::string description = HYPRSHELL_PLUGIN_DESC;
    const std::string version = HYPRSHELL_PLUGIN_VERSION;
    return {name, description, author, version};
}
