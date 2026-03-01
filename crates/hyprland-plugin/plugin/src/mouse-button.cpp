#include <hyprland/src/devices/IPointer.hpp>
#include <hyprland/src/event/EventBus.hpp>

#include "globals.h"

void onMouseButton(IPointer::SButtonEvent event, Event::SCallbackInfo &info) {
    CHECK_NO_MOUSE_BUTTON_PRESSED = false;
    // if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
    //     HyprlandAPI::addNotification(PHANDLE, "[Hyprshell Plugin] Mouse button pressed", GREEN, 4000);
    // }
}
