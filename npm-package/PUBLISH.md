# npm Package Publish Notes

This package publishes the `tamux` npm wrapper and expects GitHub release assets to already exist for the matching version.

## Pre-publish checklist

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

## Release build commands

Unix or macOS:

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
