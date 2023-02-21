#!/usr/bin/env bash

# https://daodao.zone/dao/juno13h55qdra3pg87ataedu204jtvezvs6kefqv79m5jmkx084jgzdzszzw7ky/proposals/A1
MARKET_ADDRESS=juno1uugwj8uneuvllu2e2znn2nfha0sq6n45stv6g3vg4w3v07uy2quqzxueun

JUNOD=junod
JUNO_RPC="https://juno-rpc.polkachu.com:443"

CODE_ID=`$JUNOD --node="$JUNO_RPC" \
    query wasm contract "$MARKET_ADDRESS" \
    --output=json | jq -r '.contract_info.code_id'`

$JUNOD --node="$JUNO_RPC" \
    query wasm code $CODE_ID ./artifacts/onchain_price_prediction.wasm

sha256sum ./artifacts/onchain_price_prediction.wasm
sha256sum ./artifacts/price_prediction.wasm

