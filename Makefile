.PHONY: all build test install doc

all: test

dir=$(abspath target/release)
python=env PYTHONPATH=$(dir) python3

build: 
	cargo build --release
	ln -fs $(dir)/libsnake_egg.so $(dir)/snake_egg.so 

test: test.py build
	env PYTHONPATH=$(dir) python3 test.py

install:
	maturin build
	python3 -m pip install snake_egg --force-reinstall --no-index --find-link ./target/wheels/ 

doc: build
	env PYTHONPATH=$(dir) python3 -m pydoc -w snake_egg

shell: build
	$(python) -ic 'import snake_egg'
