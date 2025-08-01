#!/bin/bash

clear

#git push origin --delete feature/insert_update
#git push origin --delete feature/insert_update_uuid

NAME="insert_update"

git checkout -b feature/${NAME}
git add .
git commit -m "Add headers for md5: ETag, If-Match, If-None-Match ; tests; authorization politics with token & workspace"
git push origin feature/${NAME}
