class Dmgr < Formula
  desc "macOS app distribution manager - archive, sign, notarize, and distribute"
  homepage "https://github.com/albertogalca/dmgr"
  version "0.0.1"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/albertogalca/dmgr/releases/download/v#{version}/dmgr-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "eeb84bda4deb653135229f3a24ceb0fcc7509e14d06eaa0ebe456cfb48329c52"
    else
      url "https://github.com/albertogalca/dmgr/releases/download/v#{version}/dmgr-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "4e4f3c6845ccc2020eb6f41604c6a167d90edbd90dd8a954ef039204dfa626d0"
    end
  end

  def install
    bin.install "dmgr"
  end

  test do
    system "#{bin}/dmgr", "--help"
  end
end
