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

function printHelp() {
  echo "Usage:"
  echo "  $FILE_NAME <command>"
  echo "Commands:"
  echo "  deploy_factory                                    transfer out ft token"
  echo "  deploy_core <core name>                           deposit out ft token"
  echo "  help                                              show help"
}

function deploy_factory() {
  echo "creating butter core factory account"
  near create-account "$FACTORY_ACCOUNT" --masterAccount $MASTER_ACCOUNT --initialBalance 15

  echo "deploying butter core factory contract"
  near deploy --accountId "$FACTORY_ACCOUNT" --wasmFile $RES_DIR/butter_core_factory.wasm
}

function deploy_core() {
  echo "create and initialize butter core contract"
  INIT_ARGS='{
                "name": "'$1'",
                "controller": "'$CONTROLLER'",
                "ref_exchange": "'$REF_EXCHANGE'",
                "wrapped_token":"'$WRAPPED_TOKEN'",
                "owner": "'$OWNER'"
              }'
  near call "$FACTORY_ACCOUNT" create_butter_core "$INIT_ARGS" --accountId $MASTER_ACCOUNT --gas 300000000000000 --deposit 10
}

if [[ $# -gt 0 ]]; then
  case $1 in
    deploy_factory)
      if [[ $# == 1 ]]; then
        deploy_factory
      else
        printHelp
        exit 1
      fi
      ;;
    deploy_core)
      if [[ $# == 2 ]]; then
        shift
        deploy_core $@
      else
        printHelp
        exit 1
      fi
      ;;
    help)
      printHelp
      ;;
    *)
      echo "Unknown command $1"
      printHelp
      exit 1
      ;;
  esac
  else
    printHelp
    exit 1
fi
