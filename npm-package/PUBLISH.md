# Publishing tamux to npm

## Prerequisites (one-time setup)

### Pre-publish checklist

1. Build release artifacts for every supported platform.
2. Confirm each platform zip includes:
   - `tamux`
   - `tamux-daemon`
   - `tamux-tui`
   - `tamux-gateway`
   - `tamux-mcp`
   - the bundled `skills/` tree at the archive root
3. Confirm each release has matching `SHA256SUMS-<platform>.txt` manifests.
4. Update `npm-package/package.json` to the release version.

### 1. npm account

```bash
# If you don't have one yet
npm adduser
# If you already have an account
npm login
```

This stores an auth token in `~/.npmrc`.

### 2. Check the package name is available

```bash
npm view tamux
```

If it returns 404, the name `tamux` is free. If taken, scope it as `@mkurman/tamux` and update the `name` field in `package.json`.

### 3. Create the GitHub repo

Create `mkurman/tamux` on GitHub (must be **public** so release assets are downloadable without auth tokens).

---

## Before each publish

### 4. Build release binaries for every platform

`install.js` expects platform zip bundles and matching checksum manifests on GitHub Releases:

```
tamux-linux-x86_64.zip
SHA256SUMS-linux-x86_64.txt
tamux-darwin-x86_64.zip
SHA256SUMS-darwin-x86_64.txt
tamux-windows-x64.zip
SHA256SUMS-windows-x64.txt
```

Each zip bundle should contain at least `tamux`, `tamux-daemon`, `tamux-tui`, `tamux-gateway`, and `tamux-mcp`. The published `SHA256SUMS-{platform}.txt` file can contain either the bundle hash or per-binary hashes; the current GitHub release workflow publishes the bundle hash.

Build with:

```bash
./scripts/build-release.sh
```

Windows from PowerShell:

```powershell
./scripts/build-release.ps1
```

Windows from WSL:

```bash
./scripts/build-release-wsl.sh
```

## Publish flow

1. Push the GitHub release assets first.
2. From `npm-package/`, run:

```bash
npm publish
```

3. Smoke-test:
   - `npm install -g tamux`
   - `tamux --help`
   - verify built-in skills were installed into `~/.tamux/skills` on Unix or `%LOCALAPPDATA%\\tamux\\skills` on Windows

## Important behavior

- The npm installer downloads the platform zip from GitHub Releases.
- The installer extracts binaries into the package `bin/` directory used by npm.
- The installer also installs bundled built-in skills into the canonical runtime skill root.
