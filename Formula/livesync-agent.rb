class LivesyncAgent < Formula
  desc "Headless bidirectional Obsidian LiveSync agent for Linux"
  homepage "https://github.com/aitorroma/obsidian-livesync"
  license "MIT"

  url "https://github.com/aitorroma/obsidian-livesync/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "be9c68a64f1375f619eb268754a8a8247d13f35044f9f5e69fe588cd00508db1"

  head "https://github.com/aitorroma/obsidian-livesync.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args(path: "./")
  end

  test do
    assert_match "livesync-agent", shell_output("#{bin}/livesync-agent --version")
  end
end
