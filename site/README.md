# cadrs site

The project page at <https://rvdende.github.io/cadrs/>: a [Proa](https://proa.so) site,
exported to static files for GitHub Pages by `.github/workflows/pages.yml`.

- `src/pages/home.rs`: the page, as HTML and as Markdown (`/index.md`).
- `src/components/`: [Proa UI](https://ui.proa.so) components, vendored with `proa ui add`.
  The page's layout follows its `hero_saas` and `features_bento` blocks.
- `styles/input.css`: Tailwind, with the Proa UI theme tokens and the cadrs blue.
- The screenshots (`/images`) and the app icon (`/icons`) are served from the repository's
  `.github/images` and `assets/icons`, so the README and the site share one copy.

## Tools

The static export (`proa export`) is on our Proa fork until it is merged upstream; the Proa
crates are patched to the same branch in `Cargo.toml`.

```sh
cargo install --locked --git https://github.com/rvdende/proa --branch feat/static-export proa-cli --bin proa
```

You also need the [Tailwind CSS CLI v4](https://github.com/tailwindlabs/tailwindcss/releases)
on your `PATH` as `tailwindcss`.

The site builds on the stable toolchain in `rust-toolchain.toml`. The repository's
`.cargo/config.toml` adds nightly-only rustflags for the app, so clear them for anything you
run here with `RUSTFLAGS=`.

## Run it

```sh
RUSTFLAGS= proa dev --open
```

## Export it

```sh
./export.sh            # into site/dist, for https://rvdende.github.io/cadrs/
./export.sh out ''     # into site/out, served from the root of a domain
```

`export.sh` builds the release binary and renders it with `proa export`. The download links
point at the latest GitHub release (set `CADRS_RELEASE=v0.1.5` to choose one).
