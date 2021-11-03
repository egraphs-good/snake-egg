.PHONY: all build test

all: test

dir=$(abspath target/release)
python=env PYTHONPATH=$(dir) python3

build: 
	cargo build --release
	ln -fs $(dir)/libsnake_egg.so $(dir)/snake_egg.so 

test: test.py build
	env PYTHONPATH=$(dir)/ python3 test.py

shell: build
	$(python) -ic 'import snake_egg'
