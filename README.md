# Python bindings for [`egg`](https://github.com/egraphs-good/egg)


# Venv

The build system creates its own python
[`venv`](https://docs.python.org/3/library/venv.html).

Executing the created `./venv/bin/activate` will ready your current shell to
use snake egg.


# Installing

- Type `make install` to build and install `snake_egg` into your python installation.
  - This will reinstall over any existing `snake_egg` installation.

- You can also install using `pip` as following:
`pip install git+https://github.com/egraphs-good/snake-egg`

- If you'd like to manually install it,
  just run `maturin build` and find the wheels in `./target/wheels/`.
