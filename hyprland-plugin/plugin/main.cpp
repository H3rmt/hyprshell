#include <hyprland/src/plugins/PluginAPI.hpp>
#include <hyprland/src/devices/IKeyboard.hpp>
#include <hyprland/src/includes.hpp>

#include "globals.hpp"
#include "defs.hpp"

// Do NOT change this function.
APICALL EXPORT std::string PLUGIN_API_VERSION() {
    return HYPRLAND_API_VERSION;
}

void onOpenLayerChange(const PHLLS &window) {
//    HyprlandAPI::addNotification(PHANDLE, "AMONGUS", CHyprColor{1.0, 0.2, 0.2, 1.0}, 5000);
}

void onKeyPressTest(const std::unordered_map<std::string, std::any> &data) {
    std::string thit = "data: " + std::to_string(data.size());
    auto keyboardIt = data.find("keyboard");
    auto eventIt = data.find("event");

    if (keyboardIt != data.end() && eventIt != data.end()) {
        auto keyboard = std::any_cast<CSharedPointer<IKeyboard> >(keyboardIt->second);
        auto event = std::any_cast<IKeyboard::SKeyEvent>(eventIt->second);
        auto ab = std::to_string(event.keycode);
        HyprlandAPI::addNotification(PHANDLE, ab, CHyprColor{0.2, 1.0, 0.2, 1.0}, 5000);
    }
}

void onKeyPress(const std::unordered_map<std::string, std::any> &data) {
    const auto keyboardIt = data.find("keyboard");
    const auto eventIt = data.find("event");

    if (keyboardIt != data.end() && eventIt != data.end()) {
        // auto keyboard = std::any_cast<CSharedPointer<IKeyboard> >(keyboardIt->second);
        auto event = std::any_cast<IKeyboard::SKeyEvent>(eventIt->second);
        if constexpr (HYPRSHELL_PRINT_START == 1) {
            HyprlandAPI::addNotification(PHANDLE, "Key Pressed: " + std::to_string(event.keycode),
                                         CHyprColor{0.2, 1.0, 0.2, 1.0}, 5000);
        }
        if (event.keycode == HYPRSHELL_SWTICH_RELEASE_KEYCODE) {
            if (event.state == WL_KEYBOARD_KEY_STATE_RELEASED) {
                system(std::string(HYPRSHELL_PROGRAM_CLOSE_SWITCH).c_str());
            }
        }
        // Escape key
        if (event.keycode == 1) {
            if (event.state == WL_KEYBOARD_KEY_STATE_RELEASED) {
                system(std::string(HYPRSHELL_PROGRAM_ESCAPE).c_str());
            }
        }
    }
}


APICALL EXPORT PLUGIN_DESCRIPTION_INFO PLUGIN_INIT(HANDLE handle) {
    PHANDLE = handle;

    if constexpr (HYPRSHELL_PRINT_START == 1) {
        HyprlandAPI::addNotification(PHANDLE, "Hyprshell Plugin started", CHyprColor{0.2, 1.0, 0.2, 1.0}, 5000);
    }

    // ALWAYS add this to your plugins. It will prevent random crashes coming from
    // mismatched header versions.
    if (const std::string HASH = __hyprland_api_get_hash(); HASH != GIT_COMMIT_HASH) {
        HyprlandAPI::addNotification(PHANDLE, "[MyPlugin] Mismatched headers! Can't proceed.",
                                     CHyprColor{1.0, 0.2, 0.2, 1.0}, 5000);
        throw std::runtime_error("[MyPlugin] Version mismatch");
    }

    // clang-format off
    static auto P1 = HyprlandAPI::registerCallbackDynamic(PHANDLE, "openLayer",[&](void*, SCallbackInfo&, const std::any &data) { onOpenLayerChange(std::any_cast<PHLLS>(data)); });
    static auto P2 = HyprlandAPI::registerCallbackDynamic(PHANDLE, "keyPress",[&](void*, SCallbackInfo&, const std::any &data) { onKeyPress(std::any_cast<std::unordered_map<std::string, std::any>>(data)); });
    // clang-format on

    const std::string name = HYPRSHELL_PLUGIN_NAME;
    const std::string author = HYPRSHELL_PLUGIN_AUTHOR;
    const std::string description = HYPRSHELL_PLUGIN_DESC;
    const std::string version = HYPRSHELL_PLUGIN_VERSION;
    return {name, description, author, version};
}

APICALL EXPORT void PLUGIN_EXIT() {
}
