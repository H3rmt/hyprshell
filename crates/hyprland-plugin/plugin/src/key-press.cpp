#include <strings.h>
#include <hyprland/src/plugins/PluginAPI.hpp>
#include <hyprland/src/devices/IKeyboard.hpp>
#include <hyprland/src/managers/input/InputManager.hpp>

#include "globals.h"
#include "defs.h"
#include "send.h"

// modifier must pre pressed and released without any other keys pressed in between
bool last_press_was_mod_press = false;

void onKeyPress(const std::unordered_map<std::string, std::any> &data, SCallbackInfo &info) {
    const auto keyboardIt = data.find("keyboard");
    const auto eventIt = data.find("event");

    if (keyboardIt != data.end() && eventIt != data.end()) {
        const auto keyboard = std::any_cast<CSharedPointer<IKeyboard> >(keyboardIt->second);
        if (g_pInputManager->shouldIgnoreVirtualKeyboard(keyboard)) {
            return;
        }
        const auto event = std::any_cast<IKeyboard::SKeyEvent>(eventIt->second);
        const auto state = keyboard->m_xkbState;
        const uint32_t keycode = event.keycode + 8; // +8 because xkbcommon expects +8 from libinput
        const bool release = event.state == WL_KEYBOARD_KEY_STATE_RELEASED;

        const bool shiftActive = xkb_state_mod_name_is_active(state, XKB_MOD_NAME_SHIFT, XKB_STATE_MODS_EFFECTIVE) == 1;
        const bool ctrlActive = xkb_state_mod_name_is_active(state, XKB_MOD_NAME_CTRL, XKB_STATE_MODS_EFFECTIVE) == 1;
        const bool superActive = xkb_state_mod_name_is_active(state, XKB_MOD_NAME_LOGO, XKB_STATE_MODS_EFFECTIVE) == 1;
        const bool altActive = xkb_state_mod_name_is_active(state, XKB_MOD_NAME_ALT, XKB_STATE_MODS_EFFECTIVE) == 1;

        const xkb_keysym_t keysym = xkb_state_key_get_one_sym(state, keycode);

        if constexpr (HYPRSHELL_PRINT_DEBUG_DEBUG == 1) {
            char buffer[20];
            xkb_keysym_get_name(keysym, buffer, sizeof(buffer));
            const auto bigString = std::string("Name: ") + buffer +
                                   " | KeySym: " + std::to_string(keysym) +
                                   // (shiftActive ? " | Shift: Active" : "") +
                                   (ctrlActive ? " | Control: Active" : "") +
                                   (superActive ? " | Meta: Active" : "") +
                                   (altActive ? " | Alt: Active" : "") +
                                   (release ? " | State: Released" : " | State: Pressed") +
                                   (last_press_was_mod_press ? " | Last press was mod press" : "");
            HyprlandAPI::addNotification(PHANDLE, "[Hyprshell Plugin] " + bigString, GREEN, 4000);
        }

        if (keysym == OVERVIEW_KEY) {
            if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
                HyprlandAPI::addNotification(
                    PHANDLE, std::string("[Hyprshell Plugin] overview pressed??: ") + std::to_string(OVERVIEW_KEY),
                    GREEN,
                    2000);
            }
            if (OVERVIEW_KEY == XKB_KEY_Super_L || OVERVIEW_KEY == XKB_KEY_Super_R ||
                OVERVIEW_KEY == XKB_KEY_Alt_L || OVERVIEW_KEY == XKB_KEY_Alt_R ||
                OVERVIEW_KEY == XKB_KEY_Control_L || OVERVIEW_KEY == XKB_KEY_Control_R
            ) {
                if (((OVERVIEW_KEY == XKB_KEY_Super_L || OVERVIEW_KEY == XKB_KEY_Super_R) && superActive && !ctrlActive
                     && !altActive) ||
                    ((OVERVIEW_KEY == XKB_KEY_Alt_L || OVERVIEW_KEY == XKB_KEY_Alt_R) && altActive && !ctrlActive
                     && !superActive) ||
                    ((OVERVIEW_KEY == XKB_KEY_Control_L || OVERVIEW_KEY == XKB_KEY_Control_R) && ctrlActive && !
                     superActive
                     && !altActive)
                ) {
                    // open overview is only a modifier key
                    if (release && last_press_was_mod_press && CHECK_NO_MOUSE_BUTTON_PRESSED) {
                        if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
                            HyprlandAPI::addNotification(PHANDLE, "[Hyprshell Plugin] mod pressed", GREEN, 2000);
                        }
                        info.cancelled = true;
                        sendStringToHyprshellSocket(HYPRSHELL_OPEN_OVERVIEW);
                    }
                } else {
                    // between pressing and releasing the mod key, there must be
                    // no mouse click (dnd)
                    // and no other key pressed
                    last_press_was_mod_press = true;
                    CHECK_NO_MOUSE_BUTTON_PRESSED = true;
                }
            } else {
                // open overview is mod + key
                if (!release && (
                        (strcasecmp(HYPRSHELL_OVERVIEW_MOD, "Alt") == 0 && altActive && !superActive && !ctrlActive) ||
                        (strcasecmp(HYPRSHELL_OVERVIEW_MOD, "Super") == 0 && superActive && !altActive && !ctrlActive) ||
                        (strcasecmp(HYPRSHELL_OVERVIEW_MOD, "Ctrl") == 0 && ctrlActive && !superActive && !altActive))
                ) {
                    if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
                        HyprlandAPI::addNotification(PHANDLE, "[Hyprshell Plugin] mod + overview pressed", GREEN, 2000);
                    }
                    info.cancelled = true;
                    sendStringToHyprshellSocket(HYPRSHELL_OPEN_OVERVIEW);
                }
            }
        } else {
            // other key than modifier was pressed
            last_press_was_mod_press = false;
        }

        // open switch mode
        if (!release && !LAYER_VISIBLE) {
            const uint32_t modMask =
                (altActive ? HYPRSHELL_MOD_ALT : 0) |
                (ctrlActive ? HYPRSHELL_MOD_CTRL : 0) |
                (superActive ? HYPRSHELL_MOD_SUPER : 0) |
                (shiftActive ? HYPRSHELL_MOD_SHIFT : 0);
            for (const auto &bind : SWITCH_BINDS) {
                const bool isTabShift =
                    (keysym == XKB_KEY_ISO_Left_Tab && bind.key == XKB_KEY_Tab && (bind.mod_mask & HYPRSHELL_MOD_SHIFT));
                if ((keysym == bind.key || isTabShift) && modMask == bind.mod_mask) {
                    if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
                        HyprlandAPI::addNotification(PHANDLE, "[Hyprshell Plugin] switch open pressed", GREEN, 2000);
                    }
                    info.cancelled = true;
                    ACTIVE_HOLD_MASK = bind.hold_mask;
                    sendStringToHyprshellSocket(bind.command);
                    break;
                }
            }
        }

        // release switch mode
        if (release) {
            uint8_t releasedMask = 0;
            if (keysym == XKB_KEY_Alt_L || keysym == XKB_KEY_Alt_R) {
                releasedMask = HYPRSHELL_HOLD_ALT;
            } else if (keysym == XKB_KEY_Control_L || keysym == XKB_KEY_Control_R) {
                releasedMask = HYPRSHELL_HOLD_CTRL;
            } else if (keysym == XKB_KEY_Super_L || keysym == XKB_KEY_Super_R) {
                releasedMask = HYPRSHELL_HOLD_SUPER;
            }
            if (releasedMask != 0 && (ACTIVE_HOLD_MASK & releasedMask) != 0) {
                if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
                    HyprlandAPI::addNotification(PHANDLE, "[Hyprshell Plugin] shift mode release pressed", GREEN, 2000);
                }
                ACTIVE_HOLD_MASK = 0;
                sendStringToHyprshellSocket(HYPRSHELL_CLOSE);
            }
        }
    }
}
