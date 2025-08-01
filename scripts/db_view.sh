#!/bin/bash

clear

CN="postgresql://root@huly.local:26257/defaultdb?sslmode=disable"

#psql "$CN" -c "SELECT * FROM hulykvs.kvs;"
psql "$CN" -c "SELECT workspace,namespace,key,convert_from(value,'UTF8') AS value, encode(md5,'hex') AS md5 \
FROM hulykvs.kvs ORDER BY namespace, key;"

