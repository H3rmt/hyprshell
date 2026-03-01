#include "globals.h"
#include "handlers.h"
#include "socket_listener.h"

PluginDescriptionInfo init(HANDLE handle) {
    PHANDLE = handle;
    // ALWAYS add this to your plugins. It will prevent random crashes coming from
    // mismatched header versions.
    const std::string COMPOSITOR_HASH = __hyprland_api_get_hash();
    const std::string CLIENT_HASH = __hyprland_api_get_client_hash();
    if (COMPOSITOR_HASH != CLIENT_HASH) {
        HyprlandAPI::addNotification(
            PHANDLE,
            "[Hyprshell Plugin] Mismatched headers! Can't proceed. (Hyprland was updated but not restarted)", RED,
            5000);
        HyprlandAPI::addNotification(PHANDLE, std::format("[Hyprshell Plugin] compositor hash: {}", COMPOSITOR_HASH),
                                     CHyprColor{1.0, 0.2, 0.2, 1.0}, 7000);
        HyprlandAPI::addNotification(PHANDLE, std::format("[Hyprshell Plugin] client hash: {}", CLIENT_HASH),
                                     CHyprColor{1.0, 0.2, 0.2, 1.0}, 7000);
        throw std::runtime_error("[Hyprshell Plugin] Version mismatch");
    }

    startSocketListener();

    // clang-format off
    static auto CB1 = HyprlandAPI::registerCallbackDynamic(PHANDLE, "configReloaded",[&](void*, SCallbackInfo&, const std::any&) { onConfigReloaded(); });
    // clang-format on

    const std::string name = "hyprshell plugin";
    const std::string author = "h3rmt";
    const std::string description = "Plugin for hyprland, used to monitor keypresses";
    // TODO inc with renovate
    const std::string version = "0.0.1";
    return {name, description, author, version};
}
