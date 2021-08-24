#include <meta/meta-plugin.h>
#include "wrapper.h"

#define COSMIC_TYPE_PLUGIN (cosmic_plugin_get_type())

G_DECLARE_FINAL_TYPE(CosmicPlugin, cosmic_plugin, COSMIC, PLUGIN, MetaPlugin)

#define COSMIC_PLUGIN(obj) (G_TYPE_CHECK_INSTANCE_CAST((obj), COSMIC_TYPE_PLUGIN, CosmicPlugin))

struct _CosmicPlugin {
  MetaPlugin parent;
};

G_DEFINE_TYPE(CosmicPlugin, cosmic_plugin, META_TYPE_PLUGIN)

static void cosmic_plugin_init(CosmicPlugin *plugin) {}

static void cosmic_plugin_class_init(CosmicPluginClass *klass) {
    MetaPluginClass *plugin_class = META_PLUGIN_CLASS(klass);
    plugin_class->confirm_display_change = cosmic_plugin_confirm_display_change;
    plugin_class->destroy = cosmic_plugin_destroy;
    plugin_class->hide_tile_preview = cosmic_plugin_hide_tile_preview;
    plugin_class->kill_switch_workspace = cosmic_plugin_kill_switch_workspace;
    plugin_class->kill_window_effects = cosmic_plugin_kill_window_effects;
    plugin_class->map = cosmic_plugin_map;
    plugin_class->minimize = cosmic_plugin_minimize;
    plugin_class->plugin_info = cosmic_plugin_info;
    plugin_class->show_tile_preview = cosmic_plugin_show_tile_preview;
    plugin_class->size_changed = cosmic_plugin_size_changed;
    plugin_class->start = cosmic_plugin_start;
    plugin_class->switch_workspace = cosmic_plugin_switch_workspace;
    plugin_class->unminimize = cosmic_plugin_unminimize;
}
