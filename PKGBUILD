pkgname=cnt-license
pkgver=1.0.0
pkgrel=1
pkgdesc="A CLI tool for generating open source license files"
arch=('x86_64')
url="https://github.com/aplt-cnt/cnt-license"
license=('GPL-3.0')
depends=('glibc')
makedepends=('cargo')
source=("$pkgname-$pkgver.tar.gz::https://github.com/aplt-cnt/cnt-license/archive/v$pkgver.tar.gz")
sha256sums=('SKIP')

build() {
  cd "$pkgname-$pkgver"
  cargo build --release --locked
}

package() {
  cd "$pkgname-$pkgver"
  install -Dm755 "target/release/$pkgname" "$pkgdir/usr/bin/$pkgname"
  install -Dm644 LICENSE-* -t "$pkgdir/usr/share/licenses/$pkgname/"
}