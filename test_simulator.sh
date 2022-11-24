#!/bin/bash

source setup_test.sh

#echo "Running simulator..."
cd simulator; bash ./tests/resim.sh
cd simulator; bash ./tests/scrypto.sh
cd simulator; bash ./tests/manifest.sh

