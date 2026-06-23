# Draft Homebrew formula for MoonPub.
#
# This file is a maintainer template. The public tap has not been published yet,
# so README installation docs should not point users at Homebrew until this file
# is copied into qiaopengjun5162/homebrew-moonpub with real checksums.
#
# Usage:
#   brew tap qiaopengjun5162/moonpub
#   brew install moonpub
#
# To set up the tap repo:
#   1. Create a GitHub repo: qiaopengjun5162/homebrew-moonpub
#   2. Copy this file into it as Formula/moonpub.rb
#   3. Push — done. Users can `brew tap qiaopengjun5162/moonpub`

class Moonpub < Formula
  desc "Markdown → WeChat Official Account, fully automated"
  homepage "https://github.com/qiaopengjun5162/moonpub"
  license "MIT"
  version "0.4.0"

  on_macos do
    if Hardware::CPU.arm?
      odie "MoonPub v0.4.0 does not publish a native macOS ARM64 binary yet. Use the x86_64 binary under Rosetta 2 or install from source."
    else
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.0/moonpub-macos-amd64.tar.gz"
      sha256 "a4c1a6b5077aa577a5244d6a3e988cb5cdb7bba95a5a603c539cf06f5a491f13"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.0/moonpub-linux-arm64.tar.gz"
      sha256 "f659e45166f7c3ef5356949de2a817aad6a1e97fe1b4ee55279daf798a396e59"
    else
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.0/moonpub-linux-amd64.tar.gz"
      sha256 "e0bf1d59c75eab7c2191e5de4ca4a9635ff176e0bed9474c0495928194a84c70"
    end
  end

  def install
    bin.install "moonpub"
  end

  test do
    system "#{bin}/moonpub", "help"
  end
end
