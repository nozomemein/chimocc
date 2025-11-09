#! /bin/bash
SCRIPT_DIR=$(cd "$(dirname "$0")" && pwd)
cd $SCRIPT_DIR
COMPILER="$SCRIPT_DIR/../target/debug/chimocc"


run() {
    input=$1
    
    echo "$input" > tmp.c
    $COMPILER tmp.c

    cc -o tmp tmp.s

    ./tmp
}

assert() {
    expected=$1
    input=$2

    run "$input"
    actual="$?"

    if [ "$actual" -ne "$expected" ]; then
        echo "Test failed: expected $expected, got $actual for input '$input'"
        exit 1
    else
        echo "Test passed: expected $expected, got $actual for input '$input'"
    fi
}

# Test cases
assert 1 "1"
assert 0 "0"
assert 255 "255"

assert 1 "1 + 0"
assert 2 "1 + 1"
assert 97 "1 + 100 - 4"
assert 4 "1 * 2 + 8 / 4"
assert 6 "1 * 2 + 2 *8 / 4"
assert 97 "1 * 2 - 2 *8 / 4 + 99"
assert 99 "1 * (2 - 2) *8 / 4 + 99"
assert 5 "(1 - 2) * (0 - 8) - 3*1"
assert 10 "-10 + 20"
assert 10 "- -10"
assert 10 "- - +10"

assert 1 "1 == 1"
assert 0 "1 != 1"
assert 0 "1 > 2"
assert 1 "1 < 2"
assert 1 "1 <= 2"
assert 1 "1 <= 1"
assert 0 "1 >= 2"
assert 1 "1 >= 1"

assert 1 "1 == 1"
assert 0 "1 != 1"
assert 0 "(1 == 1) < (1 != 1)"
assert 0 "1 == 1 < 1 != 1"
assert 1 "2 >= 2"

assert 100 "((32 + -980) <= (-  5*4 * (-2) + 9 -997 * (-2) / (-2))) + 99"

echo "All tests passed"
