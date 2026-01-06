#pragma once
#include <hyprland/src/devices/IPointer.hpp>
#include <hyprland/src/desktop/view/LayerSurface.hpp>
#include <hyprland/src/plugins/PluginAPI.hpp>


void onKeyPress(const std::unordered_map<std::string, std::any> &data, SCallbackInfo &info);

void onOpenLayerChange(const PHLLS &window, bool open);

void onMouseButton(IPointer::SButtonEvent event);

void onKeyboardFocus(const SP<CWLSurfaceResource> &surface);
