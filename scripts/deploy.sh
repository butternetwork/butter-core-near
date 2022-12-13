set -e

SCRIPT_DIR=$(dirname $0)
RES_DIR=$SCRIPT_DIR/res

source $SCRIPT_DIR/config.sh

FACTORY_ACCOUNT=$FACTORY_NAME.$MASTER_ACCOUNT

echo $MASTER_ACCOUNT
echo $FACTORY_ACCOUNT
echo $CONTROLLER
echo $REF_EXCHANGE
echo $WRAPPED_TOKEN

INIT_ARGS='{
              "name": "'$CORE_NAME'",
              "controller": "'$CONTROLLER'",
              "ref_exchange": "'$REF_EXCHANGE'",
              "wrapped_token":"'$WRAPPED_TOKEN'",
              "owner": "'$MASTER_ACCOUNT'"
            }'

echo $INIT_ARGS

echo "creating butter core factory account"
near create-account "$FACTORY_ACCOUNT" --masterAccount $MASTER_ACCOUNT --initialBalance 15

echo "deploying butter core factory contract"
near deploy --accountId "$FACTORY_ACCOUNT" --wasmFile $RES_DIR/butter_core_factory.wasm

echo "create and initialize butter core contract"
near call "$FACTORY_ACCOUNT" create_butter_core "$INIT_ARGS" --accountId $MASTER_ACCOUNT --gas 300000000000000 --deposit 10

near create-account $CORE_ID --masterAccount $MASTER_ACCOUNT --initialBalance 10
near deploy $CORE_ID --wasmFile=res/butter_core.wasm
near call $CORE_ID new "{\"controller\": \"$CONTROLLER\", \"ref_exchange\": \"$REF_EXCHANGE\",\"wrapped_token\": \"wrap.testnet\", \"owner\":\"$MASTER_ACCOUNT\"}" --accountId $MASTER_ACCOUNT