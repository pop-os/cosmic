const { Atk, Clutter, Gio, GLib, GObject, Meta, Shell, St } = imports.gi;
const AppDisplay = imports.ui.appDisplay;
const AltTab = imports.ui.altTab;
const ExtensionUtils = imports.misc.extensionUtils;
const extension = ExtensionUtils.getCurrentExtension();
const Main = imports.ui.main;
const OverviewControls = imports.ui.overviewControls;
const PanelMenu = imports.ui.panelMenu;
const SwitcherPopup = imports.ui.switcherPopup;
const Util = imports.misc.util;
const ViewSelector = imports.ui.viewSelector;

let activities_signal_show = null;
let appMenu_signal_show = null;
let workspaces_button = null;
let applications_button = null;
let search_signal_page_changed = null;
let signal_overlay_key = null;
let original_signal_overlay_key = null;

let injections = [];

function inject(object, parameter, replacement) {
    injections.push({
        "object": object,
        "parameter": parameter,
        "value": object[parameter]
    });
    object[parameter] = replacement;
}

function with_pop_shell(callback) {
    let pop_shell = Main.extensionManager.lookup("pop-shell@system76.com");
    if (pop_shell) {
        let ext = pop_shell.stateObj.ext;
        if (ext) {
            return callback(ext);
        }
    }
}

var OVERVIEW_WORKSPACES = 0;
var OVERVIEW_APPLICATIONS = 1;
var OVERVIEW_LAUNCHER = 2;

const CLOCK_CENTER = 0;
const CLOCK_LEFT = 1;
const CLOCK_RIGHT = 2;

function overview_visible(kind) {
    if (kind == OVERVIEW_WORKSPACES) {
        if (Main.overview.visibleTarget) {
            if (Main.overview.viewSelector.getActivePage() === ViewSelector.ViewPage.WINDOWS) {
                return true;
            }
        }
    } else if (kind == OVERVIEW_APPLICATIONS) {
        if (Main.overview.visibleTarget) {
            if (Main.overview.viewSelector.getActivePage() !== ViewSelector.ViewPage.WINDOWS) {
                return true;
            }
        }
    } else if (kind == OVERVIEW_LAUNCHER) {
        if (with_pop_shell((ext) => {
            return ext.window_search.dialog.visible;
        }) === true) {
            return true;
        }
    } else {
        if (Main.overview.visibleTarget) {
            return true;
        }
    }
    return false;
}

function overview_show(kind) {
    if (kind == OVERVIEW_WORKSPACES) {
        Main.overview.viewSelector._showAppsButton.checked = false;
        Main.overview.show();
    } else if (kind == OVERVIEW_APPLICATIONS) {
        Main.overview.viewSelector._showAppsButton.checked = true;
        Main.overview.show();
    } else if (kind == OVERVIEW_LAUNCHER) {
        Main.overview.hide();
        with_pop_shell((ext) => {
            ext.tiler.exit(ext);
            ext.window_search.load_desktop_files();
            ext.window_search.open(ext);
        });
    } else {
        Main.overview.show();
    }
}

function overview_hide(kind) {
    if (kind == OVERVIEW_LAUNCHER) {
        with_pop_shell((ext) => {
            ext.exit_modes();
        });
    } else {
        Main.overview.hide();
    }
}

function overview_toggle(kind) {
    if (Main.overview.animationInProgress) {
        // prevent accidental re-show
    } else if (overview_visible(kind)) {
        overview_hide(kind);
    } else {
        overview_show(kind);
    }
}

function clock_alignment(alignment) {
    const dateMenu = Main.panel.statusArea['dateMenu'];
    const container = dateMenu.container;
    const parent = container.get_parent();
    if (parent != null) {
        parent.remove_child (container);
    }

    var banner_width = Main.panel.statusArea.dateMenu._messageList.width;
    var banner_offset = Main.layoutManager.monitors[0].width - banner_width;
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
    }
}

var CosmicTopBarButton = GObject.registerClass(
class CosmicTopBarButton extends PanelMenu.Button {
    _init(settings, kind = null) {
        super._init(0.0, null, true);
        this.accessible_role = Atk.Role.TOGGLE_BUTTON;

        /* Translators: If there is no suitable word for "Activities"
           in your language, you can use the word for "Overview". */
        let name = "Activities";
        if (kind === OVERVIEW_APPLICATIONS) {
            name = "Applications";
            settings.bind("show-applications-button", this, "visible", Gio.SettingsBindFlags.DEFAULT);
        } else if (kind == OVERVIEW_WORKSPACES) {
            name = "Workspaces";
            settings.bind("show-workspaces-button", this, "visible", Gio.SettingsBindFlags.DEFAULT);
        }
        this.name = 'panel' + name;
        this.kind = kind;

        this._label = new St.Label({ text: _(name),
                                     y_align: Clutter.ActorAlign.CENTER });
        this.add_actor(this._label);

        this.label_actor = this._label;

        Main.overview.connect('shown', () => {
            this.update();
        });
        Main.overview.connect('hidden', () => {
            this.update();
        });

		// This signal cannot be connected until Main.overview is initialized
		GLib.idle_add(GLib.PRIORITY_DEFAULT, () => {
            if (Main.overview._initCalled) {
    			Main.overview.viewSelector.connect('page-changed', () => {
    				this.update();
    			});
    			return GLib.SOURCE_REMOVE;
            } else {
                return GLib.SOURCE_CONTINUE;
            }
		});

        this._xdndTimeOut = 0;
    }

    toggle() {
        overview_toggle(this.kind);
    }

    update() {
        if (overview_visible(this.kind)) {
            this.add_style_pseudo_class('overview');
            this.add_accessible_state(Atk.StateType.CHECKED);
        } else {
            this.remove_style_pseudo_class('overview');
            this.remove_accessible_state(Atk.StateType.CHECKED);
        }
    }

    handleDragOver(source, _actor, _x, _y, _time) {
        if (source != Main.xdndHandler)
            return DND.DragMotionResult.CONTINUE;

        if (this._xdndTimeOut != 0)
            GLib.source_remove(this._xdndTimeOut);
        this._xdndTimeOut = GLib.timeout_add(GLib.PRIORITY_DEFAULT, BUTTON_DND_ACTIVATION_TIMEOUT, () => {
            this._xdndToggleOverview();
        });
        GLib.Source.set_name_by_id(this._xdndTimeOut, '[gnome-shell] this._xdndToggleOverview');

        return DND.DragMotionResult.CONTINUE;
    }

    vfunc_captured_event(event) {
        if (event.type() == Clutter.EventType.BUTTON_PRESS ||
            event.type() == Clutter.EventType.TOUCH_BEGIN) {
            if (!Main.overview.shouldToggleByCornerOrButton())
                return Clutter.EVENT_STOP;
        }
        return Clutter.EVENT_PROPAGATE;
    }

    vfunc_event(event) {
        if (event.type() == Clutter.EventType.TOUCH_END ||
            event.type() == Clutter.EventType.BUTTON_RELEASE) {
            if (Main.overview.shouldToggleByCornerOrButton())
                this.toggle();
        }

        return Clutter.EVENT_PROPAGATE;
    }

    vfunc_key_release_event(keyEvent) {
        let symbol = keyEvent.keyval;
        if (symbol == Clutter.KEY_Return || symbol == Clutter.KEY_space) {
            if (Main.overview.shouldToggleByCornerOrButton()) {
                this.toggle();
                return Clutter.EVENT_STOP;
            }
        }

        return Clutter.EVENT_PROPAGATE;
    }

    _xdndToggleOverview() {
        let [x, y] = global.get_pointer();
        let pickedActor = global.stage.get_actor_at_pos(Clutter.PickMode.REACTIVE, x, y);

        if (pickedActor == this && Main.overview.shouldToggleByCornerOrButton())
            this.toggle();

        GLib.source_remove(this._xdndTimeOut);
        this._xdndTimeOut = 0;
        return GLib.SOURCE_REMOVE;
    }
});

function workspace_picker_direction(controls, left) {
    if (left) {
        let first = controls._group.get_first_child();
        if (first != controls._thumbnailsSlider) {
            controls._thumbnailsSlider.layout.slideDirection = OverviewControls.SlideDirection.LEFT;
            controls._thumbnailsBox.remove_style_class_name('workspace-thumbnails');
            controls._thumbnailsBox.set_style_class_name('workspace-thumbnails workspace-thumbnails-left');
            controls._group.set_child_below_sibling(controls._thumbnailsSlider, first)
        }
    }
    else {
        let last = controls._group.get_last_child();
        if (last != controls._thumbnailsSlider) {
            controls._thumbnailsSlider.layout.slideDirection = OverviewControls.SlideDirection.RIGHT;
            controls._thumbnailsBox.remove_style_class_name('workspace-thumbnails workspace-thumbnails-left');
            controls._thumbnailsBox.set_style_class_name('workspace-thumbnails');
            controls._group.set_child_above_sibling(controls._thumbnailsSlider, last);
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

function init(metadata) {}

function enable() {
    // Raise first window on alt-tab
    inject(AltTab.AppSwitcherPopup.prototype, "_finish", function (timestamp) {
        let appIcon = this._items[this._selectedIndex];
        if (this._currentWindow < 0)
            Main.activateWindow(appIcon.cachedWindows[0], timestamp);
        else
            Main.activateWindow(appIcon.cachedWindows[this._currentWindow], timestamp);

        SwitcherPopup.SwitcherPopup.prototype._finish.apply(this, [timestamp]);
    });

    // Always show workspaces picker
    inject(Main.overview._overview._controls._thumbnailsSlider, "_getAlwaysZoomOut", function () {
        return true;
    });

    // Pop Shop details
    let original_rebuildMenu = AppDisplay.AppIconMenu.prototype._rebuildMenu;
    inject(AppDisplay.AppIconMenu.prototype, "_rebuildMenu", function () {
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

    // Remove app "spring"
    inject(Main.overview.viewSelector, '_animateIn', function (oldPage) {
        if (oldPage)
            oldPage.hide();

        this.emit('page-empty');

        this._activePage.show();

        this._fadePageIn();
    });
    inject(Main.overview.viewSelector, '_animateOut', function (page) {
        this._fadePageOut(page);
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

    const settings = settings_new_schema(extension.metadata["settings-schema"]);

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

    // Move workspace picker to left side (TODO: RTL)
    workspace_picker_direction(Main.overview._overview._controls, true);

    // Hide search and modify background
    Main.overview._overview._searchEntry.hide();
    // This signal cannot be connected until Main.overview is initialized
    GLib.idle_add(GLib.PRIORITY_DEFAULT, () => {
        if (Main.overview._initCalled) {
            search_signal_page_changed = Main.overview.viewSelector.connect('page-changed', () => {
                if (Main.overview.viewSelector.getActivePage() === ViewSelector.ViewPage.WINDOWS) {
                    Main.overview._overview._searchEntry.hide();
                    Main.overview._overview.remove_style_class_name("cosmic-solid-bg");
                } else {
                    Main.overview._overview._searchEntry.show();
                    Main.overview._overview.add_style_class_name("cosmic-solid-bg");
                }
            });
            return GLib.SOURCE_REMOVE;
        } else {
            return GLib.SOURCE_CONTINUE;
        }
    });

    inject(Main.overview, '_shadeBackgrounds', function () {
        // Remove the code responsible for the vignette effect
        this._backgroundGroup.get_children().forEach((background) => {
            background.brightness = 1.0;
            background.opacity = 255;

            // VERY IMPORTANT: This somehow removes the initial workspaces
            // darkening. Not sure how, but it does.
            if(background.content == undefined) {
                // Shell version 3.36
                background.vignette = false;
                background.brightness = 1.0;
            } else {
                // Shell version >= 3.38
                background.content.vignette = false;
                background.content.brightness = 1.0;
            }
        })
    });

    // This can be blank. I dunno why, but it can be ¯\_(ツ)_/¯
    inject(Main.overview, '_unshadeBackgrounds', function () {
        return true;
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

    clock_alignment(settings.get_enum("clock-alignment"));
    settings.connect("changed::clock-alignment", () => {
        clock_alignment(settings.get_enum("clock-alignment"));
    });
}

function disable() {
    // Restore applications shortcut
    const SHELL_KEYBINDINGS_SCHEMA = 'org.gnome.shell.keybindings';
    Main.wm.removeKeybinding('toggle-application-view');
    Main.wm.addKeybinding(
        'toggle-application-view',
        new Gio.Settings({ schema_id: SHELL_KEYBINDINGS_SCHEMA }),
        Meta.KeyBindingFlags.IGNORE_AUTOREPEAT,
        Shell.ActionMode.NORMAL |
        Shell.ActionMode.OVERVIEW,
        Main.overview.viewSelector._toggleAppsPage.bind(Main.overview.viewSelector)
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
    if (search_signal_page_changed !== null) {
        Main.overview.viewSelector.disconnect(search_signal_page_changed);
        search_signal_page_changed = null;
    }
    Main.overview._overview._searchEntry.show();

    // Reset background changes
    Main.overview._overview.remove_style_class_name("cosmic-solid-bg");

    // Move workspace picker to right side (TODO: RTL)
    workspace_picker_direction(Main.overview._overview._controls, false);

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

    // Enable the vignette effect for each actor
    Main.overview._backgroundGroup.get_children().forEach((actor) => {
        actor.vignette = true;
    }, null);

    // Remove injections
    let i;
    for(i in injections) {
       let injection = injections[i];
       injection["object"][injection["parameter"]] = injection["value"];
    }

    clock_alignment(CLOCK_CENTER);
}

function settings_new_schema(schema) {
    const GioSSS = Gio.SettingsSchemaSource;
    const schemaDir = extension.dir.get_child("schemas");

    let schemaSource = schemaDir.query_exists(null) ?
        GioSSS.new_from_directory(schemaDir.get_path(), GioSSS.get_default(), false) :
        GioSSS.get_default();

    const schemaObj = schemaSource.lookup(schema, true);

    if (!schemaObj) {
        throw new Error("Schema " + schema + " could not be found for extension "
            + extension.metadata.uuid + ". Please check your installation.")
    }

    return new Gio.Settings({ settings_schema: schemaObj });
}
