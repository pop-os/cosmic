const { Atk, Clutter, GLib, GObject, St } = imports.gi;
const AppDisplay = imports.ui.appDisplay;
const AltTab = imports.ui.altTab;
const Main = imports.ui.main;
const PanelMenu = imports.ui.panelMenu;
const Shell = imports.gi.Shell;
const SwitcherPopup = imports.ui.switcherPopup;
const Util = imports.misc.util;
const ViewSelector = imports.ui.viewSelector;

let activities_signal_show = null;
let appMenu_signal_show = null;
let workspaces_button = null;
let applications_button = null;

let injections = [];

function inject(object, parameter, replacement) {
    injections.push({
        "object": object,
        "parameter": parameter,
        "value": object[parameter]
    });
    object[parameter] = replacement;
}

var CosmicTopBarButton = GObject.registerClass(
class CosmicTopBarButton extends PanelMenu.Button {
    _init(page = null) {
        super._init(0.0, null, true);
        this.accessible_role = Atk.Role.TOGGLE_BUTTON;

        /* Translators: If there is no suitable word for "Activities"
           in your language, you can use the word for "Overview". */
        let name = "Activities";
        if (page === ViewSelector.ViewPage.APPS) {
            name = "Applications";
        } else if (page === ViewSelector.ViewPage.WINDOWS) {
            name = "Workspaces";
        }
        this.name = 'panel' + name;
        this.page = page;

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

    is_active() {
        if (Main.overview.visibleTarget) {
            if (this.page !== null) {
                return this.page === Main.overview.viewSelector.getActivePage();
            } else {
                return true;
            }
        } else {
            return false;
        }
    }

    toggle() {
        log(this.name + " toggle: " + this.is_active());
        if (this.is_active()) {
            Main.overview.hide();
        } else {
            if (this.page === ViewSelector.ViewPage.APPS) {
                Main.overview.viewSelector._showAppsButton.checked = true;
            } else {
                Main.overview.viewSelector._showAppsButton.checked = false;
            }
            Main.overview.show();
        }
    }

    update() {
        log(this.name + " update: " + this.is_active());
        if (this.is_active()) {
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

function init(metadata) {}

function enable() {
    // Raise first window on alt-tab
    inject(AltTab.AppSwitcherPopup.prototype, "_finish", function(timestamp) {
        let appIcon = this._items[this._selectedIndex];
        if (this._currentWindow < 0)
            Main.activateWindow(appIcon.cachedWindows[0], timestamp);
        else
            Main.activateWindow(appIcon.cachedWindows[this._currentWindow], timestamp);

        SwitcherPopup.SwitcherPopup.prototype._finish.apply(this, [timestamp]);
    });

    // Always show workspaces picker
    inject(Main.overview._overview._controls._thumbnailsSlider, "_getAlwaysZoomOut", function() {
        return true;
    });

    // Pop Shop details
    let original_rebuildMenu = AppDisplay.AppIconMenu.prototype._rebuildMenu;
    inject(AppDisplay.AppIconMenu.prototype, "_rebuildMenu", function() {
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
    activities_signal_show = Main.panel.statusArea.activities.actor.connect("show", function() {
        Main.panel.statusArea.activities.actor.hide();
    });
    Main.panel.statusArea.activities.actor.hide();

    // Hide app menu
    appMenu_signal_show = Main.panel.statusArea.appMenu.actor.connect("show", function() {
        Main.panel.statusArea.appMenu.actor.hide();
    });
    Main.panel.statusArea.appMenu.actor.hide();

    // Add workspaces button
    //TODO: this removes the curved selection corner, do we care?
    workspaces_button = new CosmicTopBarButton(ViewSelector.ViewPage.WINDOWS);
    Main.panel.addToStatusArea("cosmic_workspaces", workspaces_button, 0, "left");

    // Add applications button
    applications_button = new CosmicTopBarButton(ViewSelector.ViewPage.APPS);
    Main.panel.addToStatusArea("cosmic_applications", applications_button, 1, "left");
}

function disable() {
    // Remove applications button
    applications_button.destroy();
    applications_button = null;

    // Remove workspaces button
    workspaces_button.destroy();
    workspaces_button = null;

    // Show app menu
    Main.panel.statusArea.appMenu.actor.disconnect(appMenu_signal_show);
    appMenu_signal_show = null;
    Main.panel.statusArea.appMenu.actor.show();

    // Show activities
    Main.panel.statusArea.activities.actor.disconnect(activities_signal_show);
    activities_signal_show = null;
    Main.panel.statusArea.activities.actor.show();

    // Remove injections
    let i;
    for(i in injections) {
       let injection = injections[i];
       injection["object"][injection["parameter"]] = injection["value"];
    }
}
