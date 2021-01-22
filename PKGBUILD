# Maintainer: solnce <echo c29sbmNlQHJhdGFqY3phay5vbmU= | base64 -d>
pkgname=pacdef
pkgver=0.1.0
pkgrel=1
pkgdesc='A declarative manager of Arch packages'
url='https://github.com/steven-omaha/pacdef'
source=("$pkgname-$pkgver.tar.gz::https://github.com/steven-omaha/pacdef/archive/v$pkgver.tar.gz")
arch=('any')
license=('GPL3')
depends=('python')
sha256sums=('46c2b1b61d904618473d76d23ea708f4b7ad2c089b9e63d40c0ede503f532009')


package() {
  cd "${srcdir}/${pkgname}-${pkgver}"

  install -Dm755 pacdef "${pkgdir}/usr/bin/pacdef"
}
