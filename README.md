[![PyPI version](https://badge.fury.io/py/loro.svg)](https://badge.fury.io/py/loro)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

<h1 align="center">loro-py</h1>

<p align="center">
  <a aria-label="X" href="https://x.com/loro_dev" target="_blank">
    <img alt="" src="https://img.shields.io/badge/Twitter-black?style=for-the-badge&logo=Twitter">
  </a>
  <a aria-label="Discord-Link" href="https://discord.gg/tUsBSVfqzf" target="_blank">
    <img alt="" src="https://img.shields.io/badge/Discord-black?style=for-the-badge&logo=discord">
  </a>
</p>

Python bindings for [Loro CRDT](https://github.com/loro-dev/loro) library, powered by [PyO3](https://github.com/PyO3/pyo3).

## Features

-   High-performance CRDT operations with Rust implementation
-   Rich data types support: Text, List, Map, Tree, Movable List, Counter (WIP)
-   Python-friendly API design (WIP)

## Installation

```shell
pip install loro
```

## Quick Start

```python
from loro import LoroDoc

# Create a new document
doc = LoroDoc()
# Get a text container
text = doc.get_text("text")
# Insert text
text.insert(0, "Hello, Loro!")
doc.subscribe(lambda e: print(e))
doc.commit()
```

## Development

### Prerequisites

-   Python 3.8+
-   Rust toolchain
-   [maturin](https://github.com/PyO3/maturin)

### Setup Development Environment

1. Clone the repository

```shell
git clone https://github.com/loro-dev/loro-py
cd loro-py
```

2. Install development dependencies and run

```shell
# choose your python venv
pip install maturin
maturin develop
```

## Credits

-   [PyO3](https://github.com/PyO3/pyo3) - Rust bindings for Python
