# Python bindings for [`egg`](https://github.com/egraphs-good/egg)

# Installing

- Install [`maturin`](https://github.com/PyO3/maturin), a cool Rust/Python builder thingy.
  - Download from their site or just `pip install maturin`.
- Type `make install` to build and install `snake_egg` into your python installation.
  - This will reinstall over any existing `snake_egg` installation.
  - You may want to do this in a `virtualenv`.

If you'd like to manually install it, 
  just run `maturin build` and find the wheels in `./target/wheels/`.