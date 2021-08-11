# Maintainer: solnce <echo c29sbmNlQHJhdGFqY3phay5vbmU= | base64 -d>
pkgname=pacdef
pkgver=0.5.0
pkgrel=1
pkgdesc='declarative manager of Arch packages'
url='https://github.com/steven-omaha/pacdef'
source=("https://github.com/steven-omaha/${pkgname}/archive/refs/tags/v${pkgver}.tar.gz")
arch=('any')
license=('GPL3')
depends=('python')
checkdepends=('python-pytest' 'python-mock')
sha256sums=('380633bbd958fa53e283c2bcacc0432d2fb7fa4e5a3ee7a82179fd55371d44f2')

build() {
  cd "${srcdir}/${pkgname}-${pkgver}"
  sed -i -e "s/VERSION = 'unknown'/VERSION = '${pkgver}'/" pacdef.py
}

check() {
  cd "${srcdir}/${pkgname}-${pkgver}"
  pytest -v
}

package() {
  cd "${srcdir}/${pkgname}-${pkgver}"
  install -Dm755 pacdef.py "${pkgdir}/usr/bin/pacdef"
  install -Dm644 _completion.zsh "${pkgdir}/usr/share/zsh/site-functions/_pacdef"
}
