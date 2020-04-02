#!/bin/sh

DIR=tests/script/test1
cargo run -- -s $DIR/macro.json 
diff $DIR/output.txt $DIR/output_ok.txt
if [ "$?" -eq 0 ]
then
    echo "OK"
    exit 0
else
    echo "******************** TEST FAIL *************************"
    exit 1
fi


