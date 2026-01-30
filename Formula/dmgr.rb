class Dmgr < Formula
  desc "macOS app distribution manager - archive, sign, notarize, and distribute"
  homepage "https://github.com/albertogalca/dmgr"
  version "0.1.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/albertogalca/dmgr/releases/download/v#{version}/dmgr-#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_ARM64"
    else
      url "https://github.com/albertogalca/dmgr/releases/download/v#{version}/dmgr-#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "PLACEHOLDER_SHA256_X86_64"
    end
  end

  def install
    bin.install "dmgr"
  end

  test do
    system "#{bin}/dmgr", "--version"
  end
end
