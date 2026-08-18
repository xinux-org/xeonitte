# Initial
yaml_build = custom_target(
  'init.yml',
  input: 'init.yml.in',
  output: 'init.yml',
  console: true,
  install: true,
  install_dir: pkgconfdir / 'xeonitte',
  command: [
    ymlconvert,
    '@INPUT@',
    '@OUTPUT@'
  ]
)

# Basic
yaml_build = custom_target(
  'basic.yml',
  input: 'basic.yml.in',
  output: 'basic.yml',
  console: true,
  install: true,
  install_dir: pkgconfdir / 'xeonitte',
  command: [
    ymlconvert,
    '@INPUT@',
    '@OUTPUT@'
  ]
)

# Advanced
yaml_build = custom_target(
  'advanced.yml',
  input: 'advanced.yml.in',
  output: 'advanced.yml',
  console: true,
  install: true,
  install_dir: pkgconfdir / 'xeonitte',
  command: [
    ymlconvert,
    '@INPUT@',
    '@OUTPUT@'
  ]
)

install_data('config.yml', install_dir : pkgconfdir / 'xeonitte')

install_subdir('init', install_dir : pkgconfdir / 'xeonitte')
install_subdir('basic', install_dir : pkgconfdir / 'xeonitte')
install_subdir('advanced', install_dir : pkgconfdir / 'xeonitte')

install_subdir('configcopy', install_dir : pkgconfdir / 'xeonitte')
install_subdir('relago', install_dir : pkgconfdir / 'xeonitte')


subdir('branding/xinux')
