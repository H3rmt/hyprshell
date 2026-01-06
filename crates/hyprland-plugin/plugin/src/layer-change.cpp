#include <hyprland/src/plugins/PluginAPI.hpp>
#include <hyprland/src/desktop/view/LayerSurface.hpp>

#include "globals.h"

void onOpenLayerChange(const PHLLS &window, const bool open) {
    if (window->m_namespace.starts_with("hyprshell_")) {
        // if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
        //     HyprlandAPI::addNotification(PHANDLE, "Layer active: " + std::to_string(open), GREEN, 5000);
        // }
        LAYER_VISIBLE = open;
    }
}
