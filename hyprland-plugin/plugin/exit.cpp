#include "globals.hpp"

void exit() {
    if constexpr (HYPRSHELL_PRINT_DEBUG == 1) {
        HyprlandAPI::addNotification(PHANDLE, "[Hyprshell Plugin] Plugin deactivated", RED, 3000);
    }
}
