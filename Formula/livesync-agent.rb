class LivesyncAgent < Formula
  desc "Headless bidirectional Obsidian LiveSync agent for Linux"
  homepage "https://github.com/aitorroma/obsidian-livesync-agent"
  license "MIT"

  url "https://github.com/aitorroma/obsidian-livesync-agent/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "1ee8648769126cc7f9acd2f182826e8fc5957c4a5bfc5d92f927237a446adca9"

  head "https://github.com/aitorroma/obsidian-livesync-agent.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "./")
  end

  test do
    assert_match "livesync-agent", shell_output("#{bin}/livesync-agent --version")
  end
end
