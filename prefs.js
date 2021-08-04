const Gdk = imports.gi.Gdk;
const Gio = imports.gi.Gio;
const Gtk = imports.gi.Gtk;

const ExtensionUtils = imports.misc.extensionUtils;
const extension = ExtensionUtils.getCurrentExtension();

var { settings_new_schema } = extension.imports.settings;

let settings = null;

function open_panel() {
    const appinfo = Gio.DesktopAppInfo.new("gnome-background-panel.desktop");
    const launch_ctx = Gdk.Display.get_default().get_app_launch_context();
    appinfo.launch([], launch_ctx);
}

function init() {
    settings = settings_new_schema(extension.metadata["settings-schema"]);
}

function buildPrefsWidget() {
    const label = new Gtk.Label({
        label: "Configuration for the dock, the top bar, the workspaces overview, and\nother COSMIC components is available in the Settings application.",
        justify: Gtk.Justification.CENTER,
    });

    const button = new Gtk.Button({
        label: "Configure in Settings",
        halign: Gtk.Align.CENTER,
    });
    button.connect("clicked", open_panel);

    const box = new Gtk.Box({
        orientation: Gtk.Orientation.VERTICAL,
        spacing: 18,
        halign: Gtk.Align.CENTER,
        valign: Gtk.Align.CENTER,
    });
    box.add(label);
    box.add(button);
    box.add(new Gtk.Separator({}));
    box.add(buildNichePrefsWidget());

    box.show_all();

    return box;
}

function buildNichePrefsWidget() {
    const box = new Gtk.Box({
        orientation: Gtk.Orientation.VERTICAL,
        spacing: 9,
        halign: Gtk.Align.FILL,
        valign: Gtk.Align.CENTER,
    });
    box.add(new Gtk.Label({
        label: "<b>Advanced Settings</b>",
        justify: Gtk.Justification.CENTER,
        use_markup: true,
    }));

    const superKeyBox = new Gtk.Box({
        orientation: Gtk.Orientation.HORIZONTAL,
        spacing: 9,
        halign: Gtk.Align.FILL,
        valign: Gtk.Align.CENTER,
    });
    superKeyBox.add(new Gtk.Label({
        label: "Disable Super key action (reverts to GNOME default)",
    }));
    const superKeySwitch = new Gtk.Switch({
        active: settings.get_boolean("disable-overlay-key"),
    });
    superKeySwitch.connect("notify::active", (widget) => {
        settings.set_boolean("disable-overlay-key", widget.active);
    });
    superKeyBox.add(superKeySwitch);
    box.add(superKeyBox);

    return box;
}
