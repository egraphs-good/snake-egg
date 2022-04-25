.PHONY: all build test install doc clean

activate=. venv/bin/activate


all: test

venv:
	python3 -m venv venv
	$(activate) && python -c 'import sys; print(sys.executable)'

build: venv
	$(activate) && maturin build --release

test: tests/*.py build venv
	$(activate) && maturin develop && python tests/math.py
	$(activate) && maturin develop && python tests/prop.py
	$(activate) && maturin develop && python tests/simple.py

install: venv
	$(activate) maturin build --release && \
	  pip install snake_egg --force-reinstall --no-index --find-link ./target/wheels/

doc: venv
	$(activate) && maturin develop && python -m pydoc -w snake_egg

shell: venv
	$(activate) && maturin develop && python -ic 'import snake_egg'

clean:
	cargo clean
