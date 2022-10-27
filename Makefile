.PHONY: all build test install doc clean distclean

activate=. venv/bin/activate


all: test

venv:
	python3 -m venv venv
	$(activate) && python -m pip install maturin

build: venv
	$(activate) && maturin build --release

test: egg/tests/*.py build venv
	$(activate) && maturin develop && python egg/tests/test_math.py
	$(activate) && maturin develop && python egg/tests/test_prop.py
	$(activate) && maturin develop && python egg/tests/test_simple.py
	$(activate) && maturin develop && pip install https://codeload.github.com/kszucs/ibis/zip/refs/heads/egg && python egg/tests/test_ibis.py
	$(activate) && maturin develop && python egg/tests/test_dataclass.py

stubtest: snake_egg.pyi build venv
	$(activate) && maturin develop --extras=dev && python -m mypy.stubtest snake_egg --ignore-missing-stub

mypy: snake_egg.pyi tests/*.py build venv
	$(activate) && maturin develop --extras=dev && mypy tests

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
