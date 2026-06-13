# Homebrew formula for MoonPub
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
  version "0.3.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.3.1/moonpub-macos-arm64.tar.gz"
      sha256 "" # Fill after release: shasum -a 256 moonpub-macos-arm64.tar.gz
    else
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.3.1/moonpub-macos-amd64.tar.gz"
      sha256 "" # Fill after release
    end
  end

  on_linux do
    url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.3.1/moonpub-linux-amd64.tar.gz"
    sha256 "" # Fill after release
  end

  def install
    bin.install "moonpub"
  end

  test do
    system "#{bin}/moonpub", "help"
  end
end
