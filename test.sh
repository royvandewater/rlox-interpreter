#!/bin/bash

build(){
  local verbose=$1

  if [ $verbose -eq 0 ]; then
    echo "building..."
    cargo build
  else
    cargo build &> /dev/null
  fi
}

run_test(){
  local verbose file
  verbose=$1
  file="$2"

  if [ $verbose -eq 0 ]; then
    echo ""
    echo "==========================="
    echo "$file"
    echo "---------------------------"
    ./target/debug/rlox $file
    echo "==========================="
  else
    ./target/debug/rlox $file &> /dev/null
  fi
}


run_tests(){
  local verbose=$1

  for file in examples/*.lox; do 
    run_test $verbose "$file"
  done
}


main(){
  local verbose=1
  if [[ $1 = "-v" ]] || [[ $1 = "--verbose" ]]; then
    verbose=0
  fi

  build $verbose \
  && run_tests $verbose
}

main "$@"
