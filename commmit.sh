#!/bin/bash

clear

#git push origin --delete feature/insert_update
#git push origin --delete feature/insert_update_uuid

NAME="final_tears"

# git checkout -b feature/${NAME}
git add .
git commit -m "Fix workspace JWT"
git push origin feature/${NAME}
