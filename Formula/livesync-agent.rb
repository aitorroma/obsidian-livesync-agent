class LivesyncAgent < Formula
  desc "Headless bidirectional Obsidian LiveSync agent for Linux"
  homepage "https://github.com/aitorroma/obsidian-livesync-agent"
  license "MIT"

  url "https://github.com/aitorroma/obsidian-livesync-agent/archive/refs/tags/v0.1.3.tar.gz"
  sha256 "1ae168f0f6ee7c29b23814bd424aa1cd95b5199df7ab69bc5131a0c0d082c94b"

  head "https://github.com/aitorroma/obsidian-livesync-agent.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "./")
  end

  test do
    assert_match "livesync-agent", shell_output("#{bin}/livesync-agent --version")
  end
end
