#!/bin/bash

cat /dev/stdin | cut -d ' ' -f 1 | tr '-' '\n' | sort | uniq | awk '{print (NR - 1) " => " $0 ","}'
