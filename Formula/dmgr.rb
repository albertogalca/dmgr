class Dmgr < Formula
  desc "macOS app distribution manager - archive, sign, notarize, and distribute"
  homepage "https://github.com/albertogalca/dmgr"
  version "1.0.0"
  license "MIT"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/albertogalca/dmgr/releases/download/v#{version}/dmgr-v#{version}-aarch64-apple-darwin.tar.gz"
      sha256 "REPLACE_ARM64_SHA256"
    else
      url "https://github.com/albertogalca/dmgr/releases/download/v#{version}/dmgr-v#{version}-x86_64-apple-darwin.tar.gz"
      sha256 "REPLACE_X86_64_SHA256"
    end
  end

  def install
    bin.install "dmgr"
  end

  test do
    system "#{bin}/dmgr", "--help"
  end
end
