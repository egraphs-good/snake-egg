.PHONY: all build test install doc clean

all: test

dir=$(abspath target/release)
python=env PYTHONPATH=$(dir) python3

build:
	cargo build --release
	ln -fs $(dir)/libsnake_egg.so $(dir)/snake_egg.so

test: tests/*.py build
	env PYTHONPATH=$(dir) python3 tests/math.py
	env PYTHONPATH=$(dir) python3 tests/prop.py
	env PYTHONPATH=$(dir) python3 tests/simple.py

install:
	maturin build
	$(python) -m pip install snake_egg --force-reinstall --no-index --find-link ./target/wheels/

doc: build
	env PYTHONPATH=$(dir) $(python) -m pydoc -w snake_egg

shell: build
	$(python) -ic 'import snake_egg'

clean:
	cargo clean
