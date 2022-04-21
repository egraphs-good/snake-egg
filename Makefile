.PHONY: all build test install doc clean

dir = $(abspath target/release)
python = env PYTHONPATH=$(dir) python3

# OS detection based on #
# https://stackoverflow.com/questions/714100/os-detecting-makefile
ifeq ($(OS),Windows_NT)
	LINK_SRC = $(dir)/libsnake_egg.dll
	LINK_DST = $(dir)/snake_egg.pyd
else ifeq ($(shell uname -s),Linux)
	LINK_SRC = $(dir)/libsnake_egg.so
	LINK_DST = $(dir)/snake_egg.so
else ifeq ($(shell uname -s),Darwin)
	LINK_SRC = $(dir)/libsnake_egg.dylib
	LINK_DST = $(dir)/snake_egg.so
endif



all: test



build:
	cargo build --release
	ln -fs ${LINK_SRC} ${LINK_DST}

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
