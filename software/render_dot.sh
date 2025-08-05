#!/bin/sh

cat $@ | dot -Gmargin=0.7 '-Gbgcolor=#000000' -Gcolor=white -Gfontcolor=white -Ncolor=white -Nfontcolor=white -Ecolor=white -T png | kitty +kitten icat
