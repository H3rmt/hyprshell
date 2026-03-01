#pragma once
#include <hyprland/src/plugins/PluginAPI.hpp>

const CHyprColor RED{1.0, 0.2, 0.2, 1.0};
const CHyprColor GREEN{0.2, 1.0, 0.2, 1.0};

struct PluginDescriptionInfo {
    std::string name;
    std::string description;
    std::string author;
    std::string version;
};

inline void *PHANDLE = nullptr;

void toast(const std::string &msg);

void toastError(const std::string &msg);
