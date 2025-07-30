#!/bin/bash

clear

#git push origin --delete feature/insert_update
#git push origin --delete feature/insert_update_uuid

NAME="insert_update"

git checkout -b feature/${NAME}
git add .
git commit -m "Add /api2/insert /api2/update; fix migrate; fix uuid"
git push origin feature/${NAME}
