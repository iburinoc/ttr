#!/bin/bash

cat /dev/stdin | sort | tr '-' ' ' | tr -d '()' | awk '{print (NR - 1) " => " $0}'  | sed -e 's/^\s\+\([0-9]\+\)\t/\1 => /' | sed -e 's/\([0-9]\+\)$/: \1,/'
