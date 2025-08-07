#include <hyprland/src/plugins/PluginAPI.hpp>
#include <hyprland/src/devices/IKeyboard.hpp>
#include <hyprland/src/desktop/LayerSurface.hpp>
#include <hyprland/src/includes.hpp>
#include "defs.hpp"

inline void *PHANDLE = nullptr;

inline bool LAYER_VISIBLE = false;

void onKeyPress(const std::unordered_map<std::string, std::any> &data) {
    const auto keyboardIt = data.find("keyboard");
    const auto eventIt = data.find("event");

    if (keyboardIt != data.end() && eventIt != data.end()) {
        const auto keyboard = std::any_cast<CSharedPointer<IKeyboard> >(keyboardIt->second);
        const auto event = std::any_cast<IKeyboard::SKeyEvent>(eventIt->second);
        const auto state = keyboard->m_xkbState;
        const uint32_t keycode = event.keycode + 8; // +8 because xkbcommon expects +8 from libinput
        const bool release = event.state == WL_KEYBOARD_KEY_STATE_RELEASED;

        const bool shiftActive = xkb_state_mod_name_is_active(state, XKB_MOD_NAME_SHIFT, XKB_STATE_MODS_EFFECTIVE) == 1;
        const bool ctrlActive = xkb_state_mod_name_is_active(state, XKB_MOD_NAME_CTRL, XKB_STATE_MODS_EFFECTIVE) == 1;
        const bool altActive = xkb_state_mod_name_is_active(state, XKB_MOD_NAME_ALT, XKB_STATE_MODS_EFFECTIVE) == 1;
        const xkb_keysym_t keysym = xkb_state_key_get_one_sym(keyboard->m_xkbState, keycode);

        if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
            char buffer[20];
            xkb_keysym_get_name(keysym, buffer, sizeof(buffer));
            const auto bigString = std::string("Name: ") + buffer +
                                   " | KeySym: " + std::to_string(keysym) +
                                   (shiftActive ? " | Shift: Active" : "") +
                                   (ctrlActive ? " | Control: Active" : "") +
                                   (altActive ? " | Alt: Active" : "") +
                                   (release ? " | State: Released" : " | State: Pressed");
            HyprlandAPI::addNotification(PHANDLE, "[Hyprshell Plugin] " + bigString, GREEN, 5000);
        }

        if (release) {
            if (keysym == HYPRSHELL_SWTICH_XKB_KEY_R || keysym == HYPRSHELL_SWTICH_XKB_KEY_L) {
                system(std::string(HYPRSHELL_PROGRAM_CLOSE_SWITCH).c_str());
            }
        }
    }
}

void onOpenLayerChange(const PHLLS &window, const bool open) {
    if (window->m_namespace.starts_with("hyprshell_")) {
        HyprlandAPI::addNotification(PHANDLE, "Layer active" + std::to_string(open), GREEN, 5000);
        LAYER_VISIBLE = open;
    }
}

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

void exit() {
}

// Do NOT change this function.
APICALL EXPORT std::string PLUGIN_API_VERSION() {
    return HYPRLAND_API_VERSION;
}

APICALL EXPORT PLUGIN_DESCRIPTION_INFO PLUGIN_INIT(HANDLE handle) {
    auto [name, description, author, version] = init(handle);
    return {name, description, author, version};
}

APICALL EXPORT void PLUGIN_EXIT() {
    return exit();
}
