// Three controlled windows; log received keys to check switch-key consumption.
#include <gtk/gtk.h>
#include <stdio.h>
static gboolean key(GtkEventControllerKey *controller, guint keyval, guint code,
                    GdkModifierType state, gpointer data) {
    printf("key %u\n", keyval);
    fflush(stdout);
    return FALSE;
}
static void activate(GtkApplication *app, gpointer data) {
    for (int i = 0; i < 3; i++) {
        GtkWidget *window = gtk_application_window_new(app);
        char title[32];
        snprintf(title, sizeof(title), "switch-test-%d", i);
        gtk_window_set_title(GTK_WINDOW(window), title);
        gtk_window_set_default_size(GTK_WINDOW(window), 320, 240);
        GtkEventController *controller = gtk_event_controller_key_new();
        g_signal_connect(controller, "key-pressed", G_CALLBACK(key), NULL);
        gtk_widget_add_controller(window, controller);
        gtk_window_present(GTK_WINDOW(window));
    }
}
int main(int argc, char **argv) {
    GtkApplication *app = gtk_application_new("org.hyprshell.SwitchTest",
                                              G_APPLICATION_NON_UNIQUE);
    g_signal_connect(app, "activate", G_CALLBACK(activate), NULL);
    int result = g_application_run(G_APPLICATION(app), argc, argv);
    g_object_unref(app);
    return result;
}
