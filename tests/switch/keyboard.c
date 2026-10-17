// Private Wayland virtual keyboard. Never opens /dev/uinput.
#define _GNU_SOURCE
#include "virtual-keyboard.h"
#include <stdio.h>
#include <stdlib.h>
#include <string.h>
#include <sys/mman.h>
#include <time.h>
#include <unistd.h>
#include <wayland-client.h>
#include <xkbcommon/xkbcommon.h>
static struct wl_seat *seat;
static struct zwp_virtual_keyboard_manager_v1 *manager;
static void global(void *data, struct wl_registry *r, uint32_t name,
                   const char *interface, uint32_t version) {
    if (!strcmp(interface, "wl_seat"))
        seat = wl_registry_bind(r, name, &wl_seat_interface, 1);
    if (!strcmp(interface, "zwp_virtual_keyboard_manager_v1"))
        manager = wl_registry_bind(
            r, name, &zwp_virtual_keyboard_manager_v1_interface, 1);
}
static void removed(void *data, struct wl_registry *r, uint32_t name) {}
static const struct wl_registry_listener listener = {global, removed};
int main(void) {
    struct wl_display *display = wl_display_connect(NULL);
    if (!display)
        return 1;
    struct wl_registry *registry = wl_display_get_registry(display);
    wl_registry_add_listener(registry, &listener, NULL);
    wl_display_roundtrip(display);
    if (!seat || !manager)
        return 2;
    struct xkb_context *context = xkb_context_new(0);
    struct xkb_rule_names names = {.layout = "us"};
    struct xkb_keymap *map = xkb_keymap_new_from_names(context, &names, 0);
    struct xkb_state *state = xkb_state_new(map);
    char *text = xkb_keymap_get_as_string(map, XKB_KEYMAP_FORMAT_TEXT_V1);
    int fd = memfd_create("switch-test-keymap", 0);
    if (fd < 0 ||
        write(fd, text, strlen(text) + 1) != (ssize_t)strlen(text) + 1)
        return 3;
    struct zwp_virtual_keyboard_v1 *keyboard =
        zwp_virtual_keyboard_manager_v1_create_virtual_keyboard(manager, seat);
    zwp_virtual_keyboard_v1_keymap(keyboard, WL_KEYBOARD_KEYMAP_FORMAT_XKB_V1,
                                   fd, strlen(text) + 1);
    close(fd);
    free(text);
    wl_display_roundtrip(display);
    puts("ready");
    fflush(stdout);
    unsigned code, pressed;
    while (scanf("%u %u", &code, &pressed) == 2) {
        struct timespec now;
        clock_gettime(CLOCK_MONOTONIC, &now);
        zwp_virtual_keyboard_v1_key(
            keyboard, now.tv_sec * 1000 + now.tv_nsec / 1000000, code, pressed);
        xkb_state_update_key(state, code + 8,
                             pressed ? XKB_KEY_DOWN : XKB_KEY_UP);
        zwp_virtual_keyboard_v1_modifiers(
            keyboard, xkb_state_serialize_mods(state, XKB_STATE_MODS_DEPRESSED),
            xkb_state_serialize_mods(state, XKB_STATE_MODS_LATCHED),
            xkb_state_serialize_mods(state, XKB_STATE_MODS_LOCKED),
            xkb_state_serialize_layout(state, XKB_STATE_LAYOUT_EFFECTIVE));
        wl_display_roundtrip(display);
        puts("ok");
        fflush(stdout);
    }
    zwp_virtual_keyboard_v1_destroy(keyboard);
    wl_display_roundtrip(display);
    wl_display_disconnect(display);
    xkb_state_unref(state);
    xkb_keymap_unref(map);
    xkb_context_unref(context);
    return 0;
}
