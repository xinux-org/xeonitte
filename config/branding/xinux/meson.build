# Slides
yaml_build = custom_target(
  'slides.yml',
  input: 'slides.yml.in',
  output: 'slides.yml',
  console: true,
  install: true,
  install_dir: pkgconfdir / 'xeonitte' / 'branding' / 'xinux',
  command: [
    ymlconvert,
    '@INPUT@',
    '@OUTPUT@'
  ]
)

install_data(
  ['apps.svg', 'configuration.svg', 'nix.svg', 'xinux.svg'],
  install_dir: pkgconfdir / 'xeonitte' / 'branding' / 'xinux'
)
