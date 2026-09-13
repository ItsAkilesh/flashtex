# Website

Source for https://flash-tex.github.io/flashtex/, published to the `gh-pages`
branch by `.github/workflows/site.yml`.

The workflow runs on every published release (prereleases are skipped), on
pushes to `main` that touch `site/`, and on demand from the Actions tab. It
fills these placeholders from the release's `FlashTeX.dmg`:

| Placeholder  | Example                  |
|--------------|--------------------------|
| `{{TAG}}`    | `v0.1.1`                 |
| `{{DATE}}`   | `September 12, 2026`     |
| `{{SIZE}}`   | `13.0 MB`                |
| `{{SHA256}}` | the DMG's SHA-256 digest |

Placeholders are replaced in `index.html`, `download/index.html` and
`install.sh`. Edit the pages here, never on `gh-pages`: the next publish
overwrites that branch.

Preview locally against the latest release (needs an authenticated `gh`):

```sh
python3 site/render.py /tmp/flashtex-site
open /tmp/flashtex-site/index.html
```

A release only updates the site if it has an asset named exactly
`FlashTeX.dmg`, which is what `apps/mac/scripts/make-app.sh --dmg` produces.
