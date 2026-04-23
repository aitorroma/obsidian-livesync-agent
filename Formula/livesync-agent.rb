class LivesyncAgent < Formula
  desc "Headless bidirectional Obsidian LiveSync agent for Linux"
  homepage "https://github.com/aitorroma/obsidian-livesync-agent"
  license "MIT"

  url "https://github.com/aitorroma/obsidian-livesync-agent/archive/refs/tags/v0.1.2.tar.gz"
  sha256 "e35c3ad19244adbe6850b900ba10d179f682837f21e3ff5ded81428dcf07988a"

  head "https://github.com/aitorroma/obsidian-livesync-agent.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "./")
  end

  test do
    assert_match "livesync-agent", shell_output("#{bin}/livesync-agent --version")
  end
end
