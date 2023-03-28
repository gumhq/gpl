#!/bin/bash -n
# This script is used to deploy the program artifact to the Solana Cluster
# Usage: ./deploy.sh $PROGRAM_NAME $CLUSTER

# Check if the program name is provided
if [ -z "$1" ]; then
    echo "Please provide the program name"
    exit 1
fi

# Check if the cluster name is provided
if [ -z "$2" ]; then
    echo "Please provide the cluster name"
    exit 1
fi

# Call the correct function based on the cluster
if [ "$2" = "d" ]; then
    deploy_devnet "$1"
elif [ "$2" = "m" ]; then
    deploy_mainnet "$1"
else
    echo "Please provide a valid cluster name"
    exit 1
fi

# Function to deploy to devnet
function deploy_devnet() {
    echo "Deploying to devnet"
    goki deploy -c devnet -l target/deploy/"$1".so -p target/deploy/"$1"_keypair.json
}

# Function to deploy to mainnet
function deploy_mainnet() {
    echo "Deploying to mainnet"
    # Check if $MAINNET_AUTHORITY is set
    if [ -z "$MAINNET_AUTHORITY" ]; then
        echo "Please set the MAINNET_AUTHORITY environment variable"
        exit 1
    fi
    anchor deploy --provider.wallet $MAINNET_AUTHORITY --provider.cluster mainnet target/deploy/"$1".so
}
