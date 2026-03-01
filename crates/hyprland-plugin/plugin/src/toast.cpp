#include "globals.h"

void toast(const std::string &msg) {
    HyprlandAPI::addNotification(
        PHANDLE, "[Hyprshell Plugin] " + msg,
        GREEN, 4000);
}

void toastError(const std::string &msg) {
    HyprlandAPI::addNotification(
        PHANDLE, "[Hyprshell Plugin] " + msg,
        RED, 4000);
}
