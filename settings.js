const { Gio } = imports.gi;
const ExtensionUtils = imports.misc.extensionUtils;
const extension = ExtensionUtils.getCurrentExtension();

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
