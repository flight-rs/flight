#!/bin/bash
set -ex

cargo doc
echo "<meta http-equiv=refresh content=0;url=flight/index.html>" > target/doc/index.html
wget "https://img.shields.io/badge/doc-`git rev-parse --short HEAD`-blue.svg?style=flat-square" -O target/doc/doc_shield.svg
ghp-import -n target/doc
git push -fq https://${GH_TOKEN}@github.com/${TRAVIS_REPO_SLUG}.git gh-pages