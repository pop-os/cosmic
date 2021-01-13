const AltTab = imports.ui.altTab;
const Main = imports.ui.main;
const SwitcherPopup = imports.ui.switcherPopup;

let injections = [];

function inject(object, parameter, replacement) {
    injections.push({
        "object": object,
        "parameter": parameter,
        "value": object[parameter]
    });
    object[parameter] = replacement;
}

function alt_tab_finish(timestamp) {
    let appIcon = this._items[this._selectedIndex];
    if (this._currentWindow < 0)
        Main.activateWindow(appIcon.cachedWindows[0], timestamp);
    else
        Main.activateWindow(appIcon.cachedWindows[this._currentWindow], timestamp);

    SwitcherPopup.SwitcherPopup.prototype._finish.apply(this, [timestamp]);
}

function init(metadata) {}

function enable() {
    // Raise first window on alt-tab
    inject(AltTab.AppSwitcherPopup.prototype, "_finish", alt_tab_finish);

    // Always show workspaces picker
    inject(Main.overview._overview._controls._thumbnailsSlider, "_getAlwaysZoomOut", function() {
        return true;
    });
}

function disable() {
   let i;
   for(i in injections) {
      let injection = injections[i];
      injection["object"][injection["parameter"]] = injection["value"];
   }
}
