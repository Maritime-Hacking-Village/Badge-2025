#!/bin/sh

ssort .
isort -sl .
autoflake --remove-all-unused-imports -i -r .
isort -m 3 .
docformatter --in-place --wrap-descriptions 99 --wrap-summaries 99 --docstring-length 80 99 --recursive .
black --line-length 99 .
