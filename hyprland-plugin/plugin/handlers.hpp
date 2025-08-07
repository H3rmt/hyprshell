#pragma once
#include <hyprland/src/plugins/PluginAPI.hpp>

void onKeyPress(const std::unordered_map<std::string, std::any> &data);

void onOpenLayerChange(const PHLLS &window, bool open);
