#!/bin/sh
watchmedo shell-command --patterns="*.rs" --recursive --command='rust test ${watch_src_path};  echo "-------------------------BUILD END-----------------------"'
