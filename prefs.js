const Gdk = imports.gi.Gdk;
const Gio = imports.gi.Gio;
const Gtk = imports.gi.Gtk;

const ExtensionUtils = imports.misc.extensionUtils;
const extension = ExtensionUtils.getCurrentExtension();

var { settings_new_schema } = extension.imports.settings;

let settings = null;

const DISABLE_SUPER_SETTINGS_KEY = "disable-overlay-key";

function open_panel() {
    const appinfo = Gio.DesktopAppInfo.new("gnome-background-panel.desktop");
    const launch_ctx = Gdk.Display.get_default().get_app_launch_context();
    appinfo.launch([], launch_ctx);
}

function init() {
    settings = settings_new_schema(extension.metadata["settings-schema"]);
}

function buildPrefsWidget() {
    const ui_file = extension.dir.get_path() + "/prefs.ui";
    const ui = Gtk.Builder.new_from_file(ui_file);

    const settings_button = ui.get_object("button-settings");
    settings_button.connect("clicked", open_panel);

    const disable_super_switch = ui.get_object("switch-disable-super");
    disable_super_switch.set_state(settings.get_boolean(DISABLE_SUPER_SETTINGS_KEY));
    disable_super_switch.connect("notify::active", (widget) => {
        settings.set_boolean(DISABLE_SUPER_SETTINGS_KEY, widget.active);
    });
    settings.connect("changed", (settings, key) => {
        if (key === DISABLE_SUPER_SETTINGS_KEY) {
            disable_super_switch.set_state(settings.get_boolean(key));
        }
    });

    return ui.get_object("main-widget");
}
