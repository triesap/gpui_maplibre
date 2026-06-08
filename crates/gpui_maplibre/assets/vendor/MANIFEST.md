# Vendored MapLibre GL Runtime Assets

This directory contains the optional MapLibre GL JS/CSS runtime assets used by
the `vendored-maplibre` Cargo feature.

## Source

- Package: `maplibre-gl`
- Version: `5.13.0`
- Registry artifact: `maplibre-gl-5.13.0.tgz`
- Repository: `git://github.com/maplibre/maplibre-gl-js.git`
- License: BSD-3-Clause
- npm shasum: `625f1772f0c66396c7da2a340d9b62d067c8dd83`
- npm integrity:
  `sha512-UsIVP34rZdM4TjrjhwBAhbC3HT7AzFx9p/draiAPlLr8/THozZF6WmJnZ9ck4q94uO55z7P7zoGCh+AZVoagsQ==`

## Files

| File | Bytes | SHA-256 |
| --- | ---: | --- |
| `maplibre-gl.js` | 1015688 | `8eb4bc2a47ea76873517c9d3cd1c4c9f4172be2fa4c5aabaec038e5b92f4192f` |
| `maplibre-gl.css` | 69449 | `1dae8925ee5906e442f0fd7028dded2884381b9acd7a780e5270025e09fc7c5e` |
| `LICENSE.txt` | 5984 | `ee5fc05a0677eaf69601d2c7db0d9ecd6cc27c3abc1d0733bc9ed34707cf8ef2` |

## Update

```sh
tmp_dir=$(mktemp -d)
npm pack maplibre-gl@5.13.0 --json --pack-destination "$tmp_dir"
tar -xf "$tmp_dir/maplibre-gl-5.13.0.tgz" -C "$tmp_dir"
cp "$tmp_dir/package/dist/maplibre-gl.js" crates/gpui_maplibre/assets/vendor/maplibre-gl.js
cp "$tmp_dir/package/dist/maplibre-gl.css" crates/gpui_maplibre/assets/vendor/maplibre-gl.css
cp "$tmp_dir/package/dist/LICENSE.txt" crates/gpui_maplibre/assets/vendor/LICENSE.txt
shasum -a 256 crates/gpui_maplibre/assets/vendor/maplibre-gl.js \
  crates/gpui_maplibre/assets/vendor/maplibre-gl.css \
  crates/gpui_maplibre/assets/vendor/LICENSE.txt
wc -c crates/gpui_maplibre/assets/vendor/maplibre-gl.js \
  crates/gpui_maplibre/assets/vendor/maplibre-gl.css \
  crates/gpui_maplibre/assets/vendor/LICENSE.txt
```
