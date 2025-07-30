#!/bin/bash

clear

NAME="insert_update"

git checkout -b feature/${NAME}
git add .
git commit -m "Add /api2/insert /api2/update; fix migrate; fix uuid"
git push origin feature/${NAME}
