const Gdk = imports.gi.Gdk;
const Gio = imports.gi.Gio;
const Gtk = imports.gi.Gtk;

function open_panel() {
    const appinfo = Gio.DesktopAppInfo.new("gnome-background-panel.desktop");
    const launch_ctx = Gdk.Display.get_default().get_app_launch_context();
    appinfo.launch([], launch_ctx);
}

function init() {
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
    box.append(label);
    box.append(button);

    return box;
}
