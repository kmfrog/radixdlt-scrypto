#!/bin/bash

set -x
set -e

cd "$(dirname "$0")/.."

rev2="cargo run --bin rev2 $@ --"

# Set up environment
$rev2 reset
temp=`$rev2 new-account | tee /dev/tty | awk '/Component:|Public key:/ {print $NF}'`
account=`echo $temp | cut -d " " -f1`
account_key=`echo $temp | cut -d " " -f2`
account2=`$rev2 new-account | tee /dev/tty | awk '/Component:/ {print $NF}'`
mint_auth=`$rev2 new-resource-fixed 1 | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
resource_def=`$rev2 new-resource-mutable $mint_auth | tee /dev/tty | awk '/ResourceDef:/ {print $NF}'`
$rev2 mint 777 $resource_def $mint_auth --signers $account_key
$rev2 transfer 111 $resource_def $account2 --signers $account_key

# Test helloworld
package=`$rev2 publish ../examples/helloworld | tee /dev/tty | awk '/Package:/ {print $NF}'`
component=`$rev2 call-function $package Hello new | tee /dev/tty | awk '/Component:/ {print $NF}'`
$rev2 call-method $component free_token

# Test gumball machine
package=`$rev2 publish ../examples/gumball-machine | tee /dev/tty | awk '/Package:/ {print $NF}'`
component=`$rev2 call-function $package GumballMachine new | tee /dev/tty | awk '/Component:/ {print $NF}'`
$rev2 call-method $component get_gumball 1,030000000000000000000000000000000000000000000000000004 --signers $account_key

# Test cross component call
$rev2 publish ../examples/gumball-machine --address 01a405d3129b61e86c51c3168d553d2ffd7a3f0bd2f66b5a3e9876
package=`$rev2 publish ../examples/cross-component-call | tee /dev/tty | awk '/Package:/ {print $NF}'`
component=`$rev2 call-function $package Vendor new | tee /dev/tty | awk '/Component:/ {print $NF}' | tail -n1`
$rev2 call-method $component get_gumball 1,030000000000000000000000000000000000000000000000000004 --signers $account_key

# Export abi
$rev2 export-abi $package Vendor

# Show state
$rev2 show $package
$rev2 show $component
$rev2 show $account
$rev2 show $account2