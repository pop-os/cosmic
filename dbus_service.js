const { Gio } = imports.gi;

const IFACE = `<node>
  <interface name="com.System76.Cosmic">
    <method name="GestureLeft"/>
    <method name="GestureRight"/>
    <method name="GestureUp"/>
    <method name="GestureDown"/>
    <method name="ToggleApplications"/>
    <method name="ToggleLauncher"/>
    <method name="ToggleWorkspaces"/>
  </interface>
</node>`;

var Service = class {
    constructor() {
        this.GestureLeft = () => {};
        this.GestureRight = () => {};
        this.GestureUp = () => {};
        this.GestureDown = () => {};
        this.ToggleApplications = () => {};
        this.ToggleLauncher = () => {};
        this.ToggleWorkspaces = () => {};

        this.dbus = Gio.DBusExportedObject.wrapJSObject(IFACE, this);

        const onBusAcquired = (conn) => {
            try {
                this.dbus.export(conn, '/com/System76/Cosmic')
            } catch (why) {
                global.log(`onBusAcquired export failed: ${why}`)
            }
        };

        function onNameAcquired() { }

        function onNameLost() { }

        this.id = Gio.bus_own_name(Gio.BusType.SESSION, 'com.System76.Cosmic', Gio.BusNameOwnerFlags.NONE, onBusAcquired, onNameAcquired, onNameLost);
    }

    destroy() {
        Gio.bus_unown_name(this.id);
    }
}
