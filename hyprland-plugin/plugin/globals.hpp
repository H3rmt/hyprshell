#pragma once
#include <hyprland/src/plugins/PluginAPI.hpp>

#include "defs.hpp"

inline void *PHANDLE = nullptr;

inline bool LAYER_VISIBLE = false;

PluginDescriptionInfo init(HANDLE handle);

void exit();
