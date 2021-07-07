const { Clutter, Gio, GLib, GObject, Meta, Shell, St } = imports.gi;
const AppDisplay = imports.ui.appDisplay;
const AltTab = imports.ui.altTab;
const ExtensionUtils = imports.misc.extensionUtils;
const extension = ExtensionUtils.getCurrentExtension();
const Main = imports.ui.main;
const OverviewControls = imports.ui.overviewControls;
const SwitcherPopup = imports.ui.switcherPopup;
const Util = imports.misc.util;
const ViewSelector = imports.ui.viewSelector;

var { OVERVIEW_WORKSPACES, OVERVIEW_APPLICATIONS, OVERVIEW_LAUNCHER } = extension.imports.overview;
var { overview_visible, overview_show, overview_hide, overview_toggle } = extension.imports.overview;
var { CosmicTopBarButton } = extension.imports.topBarButton;
var { settings_new_schema } = extension.imports.settings;

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

const CLOCK_CENTER = 0;
const CLOCK_LEFT = 1;
const CLOCK_RIGHT = 2;

let indicatorPad = null;
function clock_alignment(alignment) {
    // Clock Alignement breaks Date Menu, when other extensions like Dash2Panel are used
    let dash2Panel = Main.extensionManager.lookup("dash-to-panel@jderose9.github.com");
    if(dash2Panel && dash2Panel.state == ExtensionUtils.ExtensionState.ENABLED){
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

function workspace_picker_direction(controls, left) {
    if (left) {
        let first = controls._group.get_first_child();
        if (first != controls._thumbnailsSlider) {
            controls._thumbnailsSlider.layout.slideDirection = OverviewControls.SlideDirection.LEFT;
            controls._thumbnailsBox.add_style_class_name('workspace-thumbnails-left');
            controls._group.set_child_below_sibling(controls._thumbnailsSlider, first)
        }
    } else {
        let last = controls._group.get_last_child();
        if (last != controls._thumbnailsSlider) {
            controls._thumbnailsSlider.layout.slideDirection = OverviewControls.SlideDirection.RIGHT;
            controls._thumbnailsBox.remove_style_class_name('workspace-thumbnails-left');
            controls._group.set_child_above_sibling(controls._thumbnailsSlider, last);
        }
    }

    const handler_id = Main.overview.connect('showing', () => {
        Main.overview.viewSelector._workspacesDisplay._updateWorkspacesActualGeometry();
        Main.overview.disconnect(handler_id);
    });
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

function shell_theme_is_pop() {
    const stylesheet = Main.getThemeStylesheet();
    if (stylesheet)
        return stylesheet.get_path().startsWith("/usr/share/themes/Pop");
    else
        return true;
}

function show_overview_backgrounds() {
    Main.overview._backgroundGroup.get_children().forEach(background => {
        background.visible = true;
    });
}

function hide_primary_overview_backgrounds() {
    const is_pop = shell_theme_is_pop();
    Main.overview._backgroundGroup.get_children().forEach(background => {
        background.visible = !is_pop || (background.monitor != Main.layoutManager.primaryIndex);
    });
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

    // Hide search and modify background
    // This signal cannot be connected until Main.overview is initialized
    GLib.idle_add(GLib.PRIORITY_DEFAULT, () => {
        if (Main.overview._initCalled) {
            search_signal_page_changed = Main.overview.viewSelector.connect('page-changed', () => {
                Main.layoutManager._updateVisibility();

                if (Main.overview.viewSelector.getActivePage() === ViewSelector.ViewPage.WINDOWS) {
                    Main.overview._overview._searchEntry.opacity = 0;
                    Main.overview._overview._searchEntry.reactive = false;
                    Main.overview._overview.remove_style_class_name("cosmic-solid-bg");
                    show_overview_backgrounds();
                } else {
                    Main.overview._overview._searchEntry.opacity = 255;
                    Main.overview._overview._searchEntry.reactive = true;
                    Main.overview._overview.add_style_class_name("cosmic-solid-bg");
                    hide_primary_overview_backgrounds();
                }
            });
            return GLib.SOURCE_REMOVE;
        } else {
            return GLib.SOURCE_CONTINUE;
        }
    });

    // Exit from overview on Esc of applications view
    inject(Main.overview.viewSelector, '_onStageKeyPress', function (actor, event) {
        if (Main.modalCount > 1)
            return Clutter.EVENT_PROPAGATE;

        let symbol = event.get_key_symbol();

        if (symbol === Clutter.KEY_Escape) {
            if (this._searchActive) this.reset();
            Main.overview.hide();
            return Clutter.EVENT_STOP;
        } else if (this._shouldTriggerSearch(symbol)) {
            if (this._activePage === this._appsPage) this.startSearch(event);
        } else if (!this._searchActive && !global.stage.key_focus) {
            if (symbol === Clutter.KEY_Tab || symbol === Clutter.KEY_Down) {
                this._activePage.navigate_focus(null, St.DirectionType.TAB_FORWARD, false);
                return Clutter.EVENT_STOP;
            } else if (symbol === Clutter.KEY_ISO_Left_Tab) {
                this._activePage.navigate_focus(null, St.DirectionType.TAB_BACKWARD, false);
                return Clutter.EVENT_STOP;
            }
        }
        return Clutter.EVENT_PROPAGATE;
      });

    inject(Main.overview.viewSelector, 'animateFromOverview', function () {
        this._workspacesPage.opacity = 255;

        this._workspacesDisplay.animateFromOverview(this._activePage != this._workspacesPage);

        // Don't show background while animating out of applications
        this.block_signal_handler(search_signal_page_changed);
        this._showAppsButton.checked = false;
        this.unblock_signal_handler(search_signal_page_changed);

        if (!this._workspacesDisplay.activeWorkspaceHasMaximizedWindows())
            Main.overview.fadeInDesktop();
    });

    inject(Main.overview, '_shadeBackgrounds', function () {
        // Give Applications a transparent background so it can fade in
        if (Main.overview.viewSelector.getActivePage() == ViewSelector.ViewPage.APPS) {
            hide_primary_overview_backgrounds();
        } else {
            show_overview_backgrounds();
        }

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

    inject(Main.layoutManager, "_updateVisibility", function () {
        let windowsVisible = (Main.sessionMode.hasWindows && !this._inOverview) || Main.overview.viewSelector._showAppsButton.checked;

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

    clock_alignment(settings.get_enum("clock-alignment"));
    settings.connect("changed::clock-alignment", () => {
        clock_alignment(settings.get_enum("clock-alignment"));
    });

    // Move workspace picker to left side (TODO: RTL)
    workspace_picker_direction(Main.overview._overview._controls, settings.get_boolean("workspace-picker-left"));
    settings.connect("changed::workspace-picker-left", () => {
        workspace_picker_direction(Main.overview._overview._controls, settings.get_boolean("workspace-picker-left"));
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
