.PHONY: all build test install doc clean distclean

activate=. venv/bin/activate


all: test

venv:
	python3 -m venv venv
	$(activate) && python -m pip install maturin

build: venv
	$(activate) && maturin build --release

test: egg/tests/*.py build venv
	$(activate) && maturin develop && python egg/tests/math.py
	$(activate) && maturin develop && python egg/tests/prop.py
	$(activate) && maturin develop && python egg/tests/simple.py
	$(activate) && maturin develop && python egg/tests/ibis.py
	$(activate) && maturin develop && python egg/tests/dataclasses.py

install: venv
	$(activate) maturin build --release && \
	  python -m pip install snake_egg --force-reinstall --no-index \
	  --find-link ./target/wheels/

doc: venv
	$(activate) && maturin develop && python -m pydoc -w snake_egg

shell: venv
	$(activate) && maturin develop && python -ic 'import snake_egg'

clean:
	cargo clean

distclean: clean
	$(RM) -r venv
