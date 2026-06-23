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
  desc "Markdown to WeChat Official Account publishing copilot"
  homepage "https://github.com/qiaopengjun5162/moonpub"
  license "MIT"
  version "0.4.1"

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-macos-arm64.tar.gz"
      sha256 "TO_BE_FILLED_AFTER_V0_4_1_RELEASE"
    else
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-macos-amd64.tar.gz"
      sha256 "TO_BE_FILLED_AFTER_V0_4_1_RELEASE"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-linux-arm64.tar.gz"
      sha256 "TO_BE_FILLED_AFTER_V0_4_1_RELEASE"
    else
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-linux-amd64.tar.gz"
      sha256 "TO_BE_FILLED_AFTER_V0_4_1_RELEASE"
    end
  end

  def install
    bin.install "moonpub"
  end

  test do
    system "#{bin}/moonpub", "help"
  end
end
