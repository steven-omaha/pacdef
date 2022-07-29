from setuptools import setup

setup(
    name="pacdef",
    python_requires=">=3.10",
    version="0.8.0",
    packages=["pacdef"],
    # package_dir={"": "pacdef"},
    url="https://github.com/steven-omaha/pacdef",
    license="GPLv3",
    author="steven-omaha",
    author_email="35634100+steven-omaha@users.noreply.github.com",
    description="declarative package manager for Arch Linux",
    entry_points={'console_scripts': 'pacdef=pacdef.main:main'},
    install_requires=['pyalpm'],
)
