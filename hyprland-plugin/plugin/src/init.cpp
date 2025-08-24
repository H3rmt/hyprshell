#include "globals.h"
#include "handlers.h"
#include "defs.h"

PluginDescriptionInfo init(HANDLE handle) {
    PHANDLE = handle;
    // ALWAYS add this to your plugins. It will prevent random crashes coming from
    // mismatched header versions.
    if (const std::string HASH = __hyprland_api_get_hash(); HASH != GIT_COMMIT_HASH) {
        HyprlandAPI::addNotification(
            PHANDLE,
            "[Hyprshell Plugin] Mismatched headers! Can't proceed. (Hyprland was updated but not restarted)", RED,
            5000);
        throw std::runtime_error("[Hyprshell Plugin] Version mismatch");
    }

    OVERVIEW_KEY = xkb_keysym_from_name(HYPRSHELL_OVERVIEW_KEY, XKB_KEYSYM_CASE_INSENSITIVE);
    if (OVERVIEW_KEY == XKB_KEY_NoSymbol) {
        HyprlandAPI::addNotification(
            PHANDLE, std::string("[Hyprshell Plugin] Invalid overview key ") + HYPRSHELL_OVERVIEW_KEY, RED, 5000);
        throw std::runtime_error("[Hyprshell Plugin] Invalid overview key");
    }

    if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
        const auto info = std::string("Config: ") +
                          HYPRSHELL_OVERVIEW_KEY + ", " +
                          std::to_string(OVERVIEW_KEY) + ", " +
                          HYPRSHELL_SOCKET_PATH + ", " +
                          HYPRSHELL_OVERVIEW_MOD + ", " +
                          std::to_string(HYPRSHELL_SWTICH_XKB_MOD_L) + ", " +
                          std::to_string(HYPRSHELL_SWTICH_XKB_MOD_R) + ", " +
                          HYPRSHELL_OPEN_SWITCH + ", " +
                          HYPRSHELL_OPEN_SWITCH_REVERSE;
        HyprlandAPI::addNotification(PHANDLE, "[Hyprshell Plugin] Plugin started " + info, GREEN, 8000);
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
