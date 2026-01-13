project(
  'xeonitte',
  version: '0.0.3',
  meson_version: '>= 0.59',
  license: 'GPL-3.0',
)

i18n = import('i18n')
gnome = import('gnome')

base_id = 'org.xinux.Xeonitte'

dependency('openssl', version: '>= 1.0')
dependency('glib-2.0', version: '>= 2.66')
dependency('gio-2.0', version: '>= 2.66')
dependency('gtk4', version: '>= 4.6.0')
dependency('libadwaita-1', version: '>=1.2.0')
dependency('polkit-gobject-1', version: '>= 0.103')

glib_compile_resources = find_program('glib-compile-resources', required: true)
glib_compile_schemas = find_program('glib-compile-schemas', required: true)
desktop_file_validate = find_program('desktop-file-validate', required: false)
appstream_util = find_program('appstream-util', required: false)

version = meson.project_version()

prefix = get_option('prefix')
bindir = prefix / get_option('bindir')
libexecdir = prefix / get_option('libexecdir')
localedir = prefix / get_option('localedir')
sysconfdir = '/' / get_option('sysconfdir')
pkgconfdir = prefix / get_option('sysconfdir')

datadir = prefix / get_option('datadir')
pkgdatadir = datadir / meson.project_name()
iconsdir = datadir / 'icons'
podir = meson.project_source_root() / 'po'
gettext_package = meson.project_name()

profile = ''
version_suffix = ''
application_id = base_id

ymlconvert = find_program('convertyml', required : true)

subdir('data')
subdir('po')
subdir('config')

gnome.post_install(
  gtk_update_icon_cache: true,
  glib_compile_schemas: true,
  update_desktop_database: true,
)
