.PHONY: all build test install doc clean

all: test

dir=$(abspath target/release)
python=env PYTHONPATH=$(dir) python3

build:
	cargo build --release
	ln -fs $(dir)/libsnake_egg.so $(dir)/snake_egg.so

test: tests/*.py build
	$(python) tests/math.py
	$(python) tests/prop.py
	$(python) tests/simple.py

install:
	maturin build
	$(python) -m pip install snake_egg --force-reinstall --no-index --find-link ./target/wheels/

doc: build
	$(python) -m pydoc -w snake_egg

shell: build
	$(python) -ic 'import snake_egg'

clean:
	cargo clean
