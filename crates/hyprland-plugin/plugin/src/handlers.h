#pragma once
#include <hyprland/src/devices/IPointer.hpp>
#include <hyprland/src/desktop/view/LayerSurface.hpp>
#include <hyprland/src/plugins/PluginAPI.hpp>

#include <nlohmann/json.hpp>

void onConfigReloaded();
void handleMessage(const nlohmann::json& msg);
