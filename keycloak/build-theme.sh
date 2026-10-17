#!/usr/bin/env bash
# Packs the Keycloak theme into a JAR, which a Keycloak shared by several applications loads from its providers folder
# (keycloak/target/rust-axum-app-keycloak-theme.jar).
set -euo pipefail
cd "$(dirname "$0")"

name=rust-axum-app
out="$PWD/target/$name-keycloak-theme.jar"
work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

mkdir -p "$work/META-INF" "$work/theme" target
cp -R "themes/$name" "$work/theme/"
cat > "$work/META-INF/keycloak-themes.json" <<JSON
{
  "themes": [{ "name": "$name", "types": ["login", "email"] }]
}
JSON

rm -f "$out"
(cd "$work" && zip -q -r -X "$out" META-INF theme)
echo "$out"
