const { Clutter, Gio, GLib, GObject, Meta, Shell, St } = imports.gi;
const AppDisplay = imports.ui.appDisplay;
const AltTab = imports.ui.altTab;
const ExtensionUtils = imports.misc.extensionUtils;
const extension = ExtensionUtils.getCurrentExtension();
const Main = imports.ui.main;
const Overview = imports.ui.overview;
const OverviewControls = imports.ui.overviewControls;
const SwitcherPopup = imports.ui.switcherPopup;
const Util = imports.misc.util;
const WorkspacesView = imports.ui.workspacesView;
const WorkspaceThumbnail = imports.ui.workspaceThumbnail;

var applications = extension.imports.applications;
var { OVERVIEW_WORKSPACES, OVERVIEW_APPLICATIONS, OVERVIEW_LAUNCHER } = extension.imports.overview;
var { overview_visible, overview_show, overview_hide, overview_toggle } = extension.imports.overview;
var { CosmicTopBarButton } = extension.imports.topBarButton;
var { settings_new_schema } = extension.imports.settings;

let activities_signal_show = null;
let appMenu_signal_show = null;
let workspaces_button = null;
let applications_button = null;
let signal_overlay_key = null;
let signal_monitors_changed = null;
let signal_notify_checked = null;
let search_signal_showing = null;
let original_signal_overlay_key = null;
let settings = null;

let injections = [];

function inject(object, parameter, replacement) {
    injections.push({
        "object": object,
        "parameter": parameter,
        "value": object[parameter]
    });
    object[parameter] = replacement;
}

const CLOCK_CENTER = 0;
const CLOCK_LEFT = 1;
const CLOCK_RIGHT = 2;

function getWorkspacesDisplay() {
    return Main.overview._overview._controls._workspacesDisplay;
}

let indicatorPad = null;
function clock_alignment(alignment) {
    // Clock Alignement breaks Date Menu, when other extensions like Dash2Panel are used
    let dash2Panel = Main.extensionManager.lookup("dash-to-panel@jderose9.github.com");
    if(dash2Panel && dash2Panel.state == ExtensionUtils.ExtensionState.ENABLED){
        return;
    }

    if (Main.layoutManager.monitors.length == 0) {
        return;
    }

    const dateMenu = Main.panel.statusArea['dateMenu'];
    const container = dateMenu.container;
    const parent = container.get_parent();
    if (parent != null) {
        parent.remove_child (container);
    }

    const banner_width = Main.panel.statusArea.dateMenu._messageList.width;
    const banner_offset = Main.layoutManager.monitors[0].width - banner_width;
    let clock_padding = false;
    Main.messageTray._bannerBin.width = banner_width;
    if (alignment == CLOCK_LEFT) {
        Main.panel._leftBox.insert_child_at_index(container, 0);
        Main.messageTray._bannerBin.x = -banner_offset;
    } else if (alignment == CLOCK_RIGHT) {
        Main.panel._rightBox.add_actor(container);
        Main.messageTray._bannerBin.x = banner_offset;
    } else {
        Main.panel._centerBox.add_actor(container);
        Main.messageTray._bannerBin.x = 0;
        clock_padding = true;
    }

    const dateMenuBox = dateMenu.get_child_at_index(0);
    if (indicatorPad == null) {
        indicatorPad = dateMenuBox.get_child_at_index(0);
    }
    if (clock_padding) {
        if (indicatorPad.get_parent() == null) {
            dateMenuBox.insert_child_at_index(indicatorPad, 0);
        }
    } else {
        if (indicatorPad.get_parent() != null) {
            dateMenuBox.remove_child(indicatorPad);
        }
    }
}

var overlay_key_action = OVERVIEW_LAUNCHER;

function overlay_key() {
    overview_toggle(overlay_key_action);
}

function overlay_key_changed(settings) {
    if (overview_visible(overlay_key_action)) {
        overview_hide(overlay_key_action);
    }
    overlay_key_action = settings.get_enum("overlay-key-action");
}


function switch_workspace(direction) {
    // Adapted from _showWorkspaceSwitcher
    let workspaceManager = global.display.get_workspace_manager();

    // Do not switch if workspaces are disabled
    if (!Main.sessionMode.hasWorkspaces) {
        return;
    }

    // Do not switch if there is only one workspace
    if (workspaceManager.n_workspaces == 1) {
        return;
    }

    // Do not switch if workspaces are vertical but direction is not
    if (workspaceManager.layout_rows == -1 &&
        direction != Meta.MotionDirection.UP &&
        direction != Meta.MotionDirection.DOWN) {
        return;
    }

    // Do not switch if workspaces are horizontal but direction is not
    if (workspaceManager.layout_columns == -1 &&
        direction != Meta.MotionDirection.LEFT &&
        direction != Meta.MotionDirection.RIGHT) {
        return;
    }

    // Find active workspace and new workspace in switch direction
    let activeWorkspace = workspaceManager.get_active_workspace();
    let newWorkspace = activeWorkspace.get_neighbor(direction);

    // If the new workspace is different from the active one, switch to it
    if (newWorkspace != activeWorkspace) {
        newWorkspace.activate(global.get_current_time());
    }
}

var GESTURE_UP = 0;
var GESTURE_DOWN = 1;
var GESTURE_LEFT = 2;
var GESTURE_RIGHT = 3;

function gesture(kind) {
    if (kind === GESTURE_UP) {
        switch_workspace(Meta.MotionDirection.UP);
    } else if (kind === GESTURE_DOWN) {
        switch_workspace(Meta.MotionDirection.DOWN);
    } else if (kind === GESTURE_LEFT) {
        if (overview_visible(OVERVIEW_WORKSPACES)) {
            overview_hide(OVERVIEW_WORKSPACES);
        } else if (overview_visible(OVERVIEW_APPLICATIONS)) {
            overview_hide(OVERVIEW_APPLICATIONS);
        } else {
            overview_show(OVERVIEW_WORKSPACES);
        }
    } else if (kind === GESTURE_RIGHT) {
        if (overview_visible(OVERVIEW_WORKSPACES)) {
            overview_hide(OVERVIEW_WORKSPACES);
        } else if (overview_visible(OVERVIEW_APPLICATIONS)) {
            overview_hide(OVERVIEW_APPLICATIONS);
        } else {
            overview_show(OVERVIEW_APPLICATIONS);
        }
    }
}

function monitors_changed() {
    clock_alignment(settings.get_enum("clock-alignment"));
}

function gnome_40_enable() {
    // No overview at start-up
    if (Main.layoutManager._startingUp && Main.sessionMode.hasOverview) {
        inject(Main.sessionMode, "hasOverview", false);
        Main.layoutManager.connect('startup-complete', () => {
            Main.sessionMode.hasOverview = true;
        });
    }

    // Disable type to search
    inject(Main.overview._overview._controls._searchController, '_onStageKeyPress', function (actor, event) {
        if (Main.modalCount > 1)
            return Clutter.EVENT_PROPAGATE;

        let symbol = event.get_key_symbol();

        if (symbol === Clutter.KEY_Escape) {
            Main.overview.hide();
            return Clutter.EVENT_STOP;
        }
        return Clutter.EVENT_PROPAGATE;
    });

    applications.enable();

    const overview_show = Main.overview.show;
    inject(Main.overview, 'show', function() {
        overview_show.call(this);
        applications.hide();
    });

    const overview_hide = Main.overview.hide;
    inject(Main.overview, 'hide', function() {
        overview_hide.call(this);
        applications.hide();
    });
}

function gnome_40_disable() {
    applications.disable();
}

function init(metadata) {}

function enable() {
    gnome_40_enable();

    // Raise first window on alt-tab
    inject(AltTab.AppSwitcherPopup.prototype, "_finish", function (timestamp) {
        let appIcon = this._items[this._selectedIndex];
        if (this._currentWindow < 0)
            Main.activateWindow(appIcon.cachedWindows[0], timestamp);
        else
            Main.activateWindow(appIcon.cachedWindows[this._currentWindow], timestamp);

        SwitcherPopup.SwitcherPopup.prototype._finish.apply(this, [timestamp]);
    });

    // Pop Shop details
    let AppMenu;
    if (AppDisplay.AppIconMenu !== undefined) {
        AppMenu = AppDisplay.AppIconMenu;
    } else {
        AppMenu = AppDisplay.AppMenu;
    }
    let original_rebuildMenu = AppMenu.prototype._rebuildMenu;
    inject(AppMenu.prototype, "_rebuildMenu", function () {
        let ret = original_rebuildMenu.apply(this, arguments);

        if (!this._source.app.is_window_backed()) {
            if (Shell.AppSystem.get_default().lookup_app('io.elementary.appcenter.desktop')) {
                this._appendSeparator();
                let item = this._appendMenuItem(_("Show Details"));
                item.connect('activate', () => {
                    let id = this._source.app.get_id();
                    Util.trySpawn(["io.elementary.appcenter", "appstream://" + id]);
                    Main.overview.hide();
                });
            }
        }

        return ret;
    });

    // Hide activities button
    activities_signal_show = Main.panel.statusArea.activities.connect("show", function() {
        Main.panel.statusArea.activities.hide();
    });
    Main.panel.statusArea.activities.hide();

    // Hide app menu
    appMenu_signal_show = Main.panel.statusArea.appMenu.connect("show", function() {
        Main.panel.statusArea.appMenu.hide();
    });
    Main.panel.statusArea.appMenu.hide();

    settings = settings_new_schema(extension.metadata["settings-schema"]);

    // Load overlay key action and keep it up to date with settings
    overlay_key_changed(settings);
    settings.connect("changed::overlay-key-action", () => {
        overlay_key_changed(settings);
    });

    // Add workspaces button
    //TODO: this removes the curved selection corner, do we care?
    workspaces_button = new CosmicTopBarButton(settings, OVERVIEW_WORKSPACES);
    Main.panel.addToStatusArea("cosmic_workspaces", workspaces_button, 0, "left");

    // Add applications button
    applications_button = new CosmicTopBarButton(settings, OVERVIEW_APPLICATIONS);
    Main.panel.addToStatusArea("cosmic_applications", applications_button, 1, "left");

    // Hide search
    // This signal cannot be connected until Main.overview is initialized
    GLib.idle_add(GLib.PRIORITY_DEFAULT, () => {
        if (!Main.overview._initCalled)
            return GLib.SOURCE_CONTINUE;

        Main.overview.searchEntry.hide();

        return GLib.SOURCE_REMOVE;
    });

    inject(Main.layoutManager, "_updateVisibility", function () {
        let windowsVisible = (Main.sessionMode.hasWindows && !this._inOverview) || Main.overview.dash.showAppsButton.checked;

        global.window_group.visible = windowsVisible;
        global.top_window_group.visible = windowsVisible;

        this._trackedActors.forEach(this._updateActorVisibility.bind(this));
    });

    // Block original overlay key handler
    original_signal_overlay_key = GObject.signal_handler_find(global.display, { signalId: "overlay-key" });
    if (original_signal_overlay_key !== null) {
        global.display.block_signal_handler(original_signal_overlay_key);
    }

    // Connect modified overlay key handler
    const A11Y_SCHEMA = 'org.gnome.desktop.a11y.keyboard';
    const STICKY_KEYS_ENABLE = 'stickykeys-enable';
    let _a11ySettings = new Gio.Settings({ schema_id: A11Y_SCHEMA });
    signal_overlay_key = global.display.connect("overlay-key", () => {
        if (!_a11ySettings.get_boolean(STICKY_KEYS_ENABLE))
            overlay_key();
    });

    // Make applications shortcut hide/show overview
    const SHELL_KEYBINDINGS_SCHEMA = 'org.gnome.shell.keybindings';
    Main.wm.removeKeybinding('toggle-application-view');
    Main.wm.addKeybinding(
        'toggle-application-view',
        new Gio.Settings({ schema_id: SHELL_KEYBINDINGS_SCHEMA }),
        Meta.KeyBindingFlags.IGNORE_AUTOREPEAT,
        Shell.ActionMode.NORMAL |
        Shell.ActionMode.OVERVIEW,
        () => overview_toggle(OVERVIEW_APPLICATIONS)
    );

    settings.connect("changed::clock-alignment", () => {
        clock_alignment(settings.get_enum("clock-alignment"));
    });

    // Connect monitors-changed handler
    signal_monitors_changed = Main.layoutManager.connect('monitors-changed', monitors_changed);
    monitors_changed();
}

function disable() {
    gnome_40_disable();

    // Disconnect monitors-changed handler
    if (signal_monitors_changed !== null) {
        Main.layoutManager.disconnect(signal_monitors_changed);
        signal_monitors_changed = null;
    }

    // Restore applications shortcut
    const SHELL_KEYBINDINGS_SCHEMA = 'org.gnome.shell.keybindings';
    Main.wm.removeKeybinding('toggle-application-view');

    let obj = Main.overview._overview._controls;
    Main.wm.addKeybinding(
        'toggle-application-view',
        new Gio.Settings({ schema_id: SHELL_KEYBINDINGS_SCHEMA }),
        Meta.KeyBindingFlags.IGNORE_AUTOREPEAT,
        Shell.ActionMode.NORMAL |
        Shell.ActionMode.OVERVIEW,
        obj._toggleAppsPage.bind(obj)
    );

    // Disconnect modified overlay key handler
    if (signal_overlay_key !== null) {
        global.display.disconnect(signal_overlay_key);
        signal_overlay_key = null;
    }

    // Unblock original overlay key handler
    if (original_signal_overlay_key !== null) {
        global.display.unblock_signal_handler(original_signal_overlay_key);
        original_signal_overlay_key = null;
    }

    // Show search
    if (signal_notify_checked !== null) {
        Main.overview.dash.showAppsButton.disconnect(signal_notify_checked);
        signal_notify_checked = null;
    }
    if (search_signal_showing !== null) {
        Main.overview.disconnect(search_signal_showing);
        search_signal_showing = null;
    }
    Main.overview.searchEntry.show();

    // Reset background changes
    Main.overview._overview.remove_style_class_name("cosmic-solid-bg");

    // Remove applications button
    applications_button.destroy();
    applications_button = null;

    // Remove workspaces button
    workspaces_button.destroy();
    workspaces_button = null;

    // Show app menu
    Main.panel.statusArea.appMenu.disconnect(appMenu_signal_show);
    appMenu_signal_show = null;
    Main.panel.statusArea.appMenu.show();

    // Show activities button
    Main.panel.statusArea.activities.disconnect(activities_signal_show);
    activities_signal_show = null;
    Main.panel.statusArea.activities.show();

    // Remove injections
    let i;
    for(i in injections) {
       let injection = injections[i];
       injection["object"][injection["parameter"]] = injection["value"];
    }

    clock_alignment(CLOCK_CENTER);
}
