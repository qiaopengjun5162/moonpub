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
      sha256 "4297a10facac9feba6a8cf0d8920b129118ddb32ff042b1b416ece3c772eecfb"
    else
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-macos-amd64.tar.gz"
      sha256 "5812456d75e6e038de18bd804296a0b33213183888d415971aeab512a9aa809c"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-linux-arm64.tar.gz"
      sha256 "b07f395d9c8ce41607398522e053a006dc51c1dd5ab719d4e889d7d365d3455b"
    else
      url "https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-linux-amd64.tar.gz"
      sha256 "924de20c7a29aa6394e18bcad29fe97ed566aeef81c5510893d72efca8bff119"
    end
  end

  def install
    bin.install "moonpub"
  end

  test do
    system "#{bin}/moonpub", "help"
  end
end
