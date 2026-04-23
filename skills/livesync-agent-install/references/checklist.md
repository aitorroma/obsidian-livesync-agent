# Installation Checklist for Agents

1. Confirm plugin prerequisite (Self-hosted LiveSync).
2. Pick install path by platform:
   - Linux x86_64 -> install script
   - macOS -> Homebrew tap
3. Verify binary in PATH (`livesync-agent --version`).
4. Run `livesync-agent setup`.
5. Run `livesync-agent sync-once`.
6. Optional: `daemon --interval-seconds 30`.
