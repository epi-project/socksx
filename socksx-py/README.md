## socksx-py
The `socksx-py` crate is a [PyO3](https://github.com/PyO3/PyO3)-based Python interface to `socksx`.

You can build and install this Python package locally (requires [`pipenv`](https://github.com/pypa/pipenv) and [`maturin`](https://github.com/PyO3/maturin)):

```shell
$ pipenv install && pipenv shell
$ maturin develop -m ./Cargo.toml
```

To build a manylinux releases:

```shell
$ docker run --rm -v $(pwd):/io konstin2/maturin build --release -m ./Cargo.toml
```