#include <hyprland/src/plugins/PluginAPI.hpp>
#include <hyprland/src/devices/IKeyboard.hpp>

#include "globals.hpp"
#include "defs.hpp"


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
