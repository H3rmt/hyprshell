#pragma once
#include <format>

#include "globals.h"

#include <string>
#include <hyprland/src/plugins/PluginAPI.hpp>

void bind(const std::string& mod, const std::string& key, const std::string& command) {

    auto args = std::format("bind {},{},exec,notify-send a a", mod, key);
    HyprlandAPI::invokeHyprctlCommand("keyword", args);
}

void onConfigReloaded() {
    HyprlandAPI::addNotification(PHANDLE, "onConfigReloaded", CHyprColor{0.2, 0.0, 1.0, 1.0}, 7000);

    auto mod = "SUPER";
    auto key = "B";
    bind(mod, key, "");
}
