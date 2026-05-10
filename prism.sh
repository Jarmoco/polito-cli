#!/bin/bash
cd $HOME/Dev/PolitoCLI
HOST=$(hostname -I | awk '{print $1}')
$HOME/.local/bin/prism-cli mock -h "$HOST" -p 6509 ./openapi.yaml