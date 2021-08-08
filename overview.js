const Main = imports.ui.main;
const OverviewControls = imports.ui.overviewControls;

const GNOME_VERSION = imports.misc.config.PACKAGE_VERSION;

function with_pop_shell(callback) {
    let pop_shell = Main.extensionManager.lookup("pop-shell@system76.com");
    if (pop_shell && pop_shell.stateObj) {
        let ext = pop_shell.stateObj.ext;
        if (ext) {
            return callback(ext);
        }
    }
}

var OVERVIEW_WORKSPACES = 0;
var OVERVIEW_APPLICATIONS = 1;
var OVERVIEW_LAUNCHER = 2;

function overview_visible(kind) {
    if (kind == OVERVIEW_WORKSPACES) {
        if (Main.overview.visibleTarget) {
            if (!Main.overview.dash.showAppsButton.checked) {
                return true;
            }
        }
    } else if (kind == OVERVIEW_APPLICATIONS) {
        if (Main.overview.visibleTarget) {
            if (Main.overview.dash.showAppsButton.checked) {
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
        if (GNOME_VERSION.startsWith("3.38")) {
            Main.overview.dash.showAppsButton.checked = false;
            Main.overview.show();
        } else if (Main.overview.visible) {
            Main.overview.dash.showAppsButton.checked = false;
        } else {
            Main.overview.show(OverviewControls.ControlsState.WINDOW_PICKER);
        }
    } else if (kind == OVERVIEW_APPLICATIONS) {
        if (GNOME_VERSION.startsWith("3.38")) {
            Main.overview.dash.showAppsButton.checked = true;
            Main.overview.show();
        } else if (Main.overview.visible) {
            Main.overview.dash.showAppsButton.checked = true;
        } else {
            Main.overview.show(OverviewControls.ControlsState.APP_GRID);
        }
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
