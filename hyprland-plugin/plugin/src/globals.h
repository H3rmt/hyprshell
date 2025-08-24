#pragma once
#include <hyprland/src/plugins/PluginAPI.hpp>
#include <hyprland/src/devices/IKeyboard.hpp>

#include "defs.h"

inline void *PHANDLE = nullptr;

inline bool LAYER_VISIBLE = false;

inline xkb_keysym_t OVERVIEW_KEY;

PluginDescriptionInfo init(HANDLE handle);

void exit();
