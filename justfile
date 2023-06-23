default:
  @just --list

# Bump version, tag, and release
release VERSION:
  sed -i "/^version = /s/\"[0-9a-z.-]*\"/\"{{VERSION}}\"/" Cargo.toml
  git commit -am "chore: prepare for v{{VERSION}}"
  git tag -a v{{VERSION}} -m v{{VERSION}}
  git push
  git push origin --tags
  cargo publish
