#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

resim="cargo run --bin resim $@ --"

# Create test accounts and public keys
$resim reset
temp=`$resim new-account | tee /dev/tty | awk '/Account component address:/ {print $NF}'`
account=`echo $temp | cut -d " " -f1`
account2=`$resim new-account | tee /dev/tty | awk '/Account component address:/ {print $NF}'`

# Test - create fixed supply badge
minter_badge=`$resim new-badge-fixed 1 --name 'MintBadge' | tee /dev/tty | awk '/Resource:/ {print $NF}'`

# Test - create mutable supply token
token_address=`$resim new-token-mutable $minter_badge | tee /dev/tty | awk '/Resource:/ {print $NF}'`

# Test - mint and transfer
$resim mint 777 $token_address $minter_badge
$resim transfer 111 $token_address $account2

# Test - publish, call-funciton and call-method
package=`$resim publish ../examples/hello-world | tee /dev/tty | awk '/Package:/ {print $NF}'`
component=`$resim call-function $package Hello instantiate_hello | tee /dev/tty | awk '/Component:/ {print $NF}'`
$resim call-method $component free_token

# Test - export abi
$resim export-abi $package Hello

# Test - dump component state
$resim show $package
$resim show $component
$resim show $account
$resim show $account2
$resim show $token_address

# Test - output manifest
$resim new-badge-fixed 1 --name 'MintBadge' --manifest ./target/temp.rtm
cat ./target/temp.rtm
