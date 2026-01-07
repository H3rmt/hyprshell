#pragma once
#include <hyprland/src/plugins/PluginAPI.hpp>
#include <hyprland/src/devices/IKeyboard.hpp>
#include <cstdint>
#include <vector>

#include "defs.h"

inline void *PHANDLE = nullptr;

inline bool LAYER_VISIBLE = false;
inline bool CHECK_NO_MOUSE_BUTTON_PRESSED = false;

inline xkb_keysym_t OVERVIEW_KEY;
struct SwitchBind {
    xkb_keysym_t key;
    uint32_t mod_mask;
    uint8_t hold_mask;
    const char *command;
};

inline std::vector<SwitchBind> SWITCH_BINDS;
inline uint8_t ACTIVE_HOLD_MASK = 0;

PluginDescriptionInfo init(HANDLE handle);

void exit();
