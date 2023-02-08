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
  local exit_code verbose file
  verbose=$1
  file="$2"

  if [ $verbose -eq 0 ]; then
    echo ""
    echo "==========================="
    echo "$file"
    echo "---------------------------"
    ./target/debug/rlox $file
    exit_code=$?
    echo "==========================="
    return $exit_code
  else
    ./target/debug/rlox $file &> /dev/null
  fi
}


run_tests(){
  local verbose=$1

  for file in examples/*.lox; do
    run_test $verbose "$file"

    if [ $? -ne 0 ]; then
      run_test 0 "$file"
    fi
  done
}

run_tests_that_should_error(){
  local verbose=$1

  for file in examples/expect_error/*.lox; do
    run_test $verbose "$file"

    if [ $? -eq 0 ]; then
      echo "test $file should have errored but did not"
      run_test 0 "$file"
    fi
  done
}


main(){
  local verbose=1
  if [[ $1 = "-v" ]] || [[ $1 = "--verbose" ]]; then
    verbose=0
  fi

  build $verbose \
  && run_tests $verbose \
  && run_tests_that_should_error $verbose
}

main "$@"
