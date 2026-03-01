#include "globals.h"
#include <hyprland/src/desktop/view/LayerSurface.hpp>

void onKeyboardFocus(const SP<CWLSurfaceResource> &surface) {
    if (LAYER_VISIBLE) {
        if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
            HyprlandAPI::addNotification(PHANDLE, "Focus change", GREEN, 5000);
        }
        // TODO focus layer
    }
}
