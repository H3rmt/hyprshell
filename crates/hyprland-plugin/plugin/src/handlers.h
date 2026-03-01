#pragma once
#include <hyprland/src/devices/IPointer.hpp>
#include <hyprland/src/desktop/view/LayerSurface.hpp>
#include <hyprland/src/event/EventBus.hpp>


void onKeyPress(const IKeyboard::SKeyEvent &event, Event::SCallbackInfo &info);

void onOpenLayerChange(const PHLLS &window, bool open);

void onMouseButton(IPointer::SButtonEvent event, Event::SCallbackInfo &);

void onKeyboardFocus(const SP<CWLSurfaceResource> &surface);
