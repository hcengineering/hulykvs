#!/bin/bash

clear

URL="http://localhost:8094/api2"

R='\033[0;31m' # Color red
G='\033[0;32m' # Color green
W='\033[0;33m' # Color ?
S='\033[0;34m' # Color Blue
F='\033[0;35m' # Color Fiolet
L='\033[0;36m' # Color LightBlue
N='\033[0m' # No Color
GRAY='\033[90m' # bright black

dbview() {
    local CN="postgresql://root@huly.local:26257/defaultdb?sslmode=disable"
    local QUERY="SELECT workspace, namespace, key, convert_from(value,'UTF8') AS value, encode(md5,'hex') AS md5 FROM hulykvs.kvs ORDER BY namespace, key;"
    local result
    if [ -z "$1" ]; then psql "$CN" -c "$QUERY"
    else
        #result=$(psql "$CN" -c "$QUERY" | grep --color=always -i "$1")
        result=$(psql "$CN" -c "$QUERY" | grep -i "$1")
        if [ -z "$result" ]; then echo -e "${R} ${1} -- not found${N}"
        else echo -e "${G}${result}${N}"
        fi
    fi
}

api() {
  local tmpfile
  tmpfile=$1
  local status
  status=$(head -n 1 "$tmpfile")
  local status_code
  status_code=$(echo "$status" | awk '{print $2}')
  local etag
  etag=$(grep -i "^ETag:" "${tmpfile}")
  local body
  body=$(awk 'found { print; next } NF == 0 { found = 1 }' "$tmpfile")
  case "$status_code" in
	2*) echo -en "${G}${status}${N}" ;;
	3*) echo -en "${F}${status}${N}" ;;
	4*) echo -en "${R}${status}${N}" ;;
	5*) echo -en "${R}${status}${N}" ;;
	*)  echo -en "${GRAY}${status}${N}" ;;
  esac
  if [ -n "$etag" ]; then echo -n -e " ${F}${etag}${N}" ; fi
  if [ -n "$body" ]; then echo -e "\n   ${GRAY}[${body}]${N}" ; else echo -e " ${L}(no body)${N}" ; fi
  rm -f "$tmpfile"
}

get() {
  echo -n -e "📥 ${L}GET ${W}$1${N} > "
  local tmpfile
  tmpfile=$(mktemp)
  curl -i -s -X GET "$URL/$1" -H "Authorization: Bearer ${TOKEN}" | tr -d '\r' > "$tmpfile"
  api ${tmpfile}
}

put() { # If-None-Match If-Match
  local match
  local match_prn
  if [ -n "$3" ]; then match=(-H "$3: $4") ; else match=() ; fi
  if [ -n "$3" ]; then match_prn=" ${F}$3:$4${N}" ; else match_prn="" ; fi
  echo -n -e "📥 ${L}PUT ${W}$1${N}${match_prn} > "
  local tmpfile
  tmpfile=$(mktemp)
  curl -i -s -X PUT "$URL/$1" -H "Authorization: Bearer ${TOKEN}" "${match[@]}" -H "Content-Type: application/json" -d "$2" | tr -d '\r' > "$tmpfile"
  api ${tmpfile}
}

delete() {
  echo -n -e "📥 ${L}DELETE ${W}$1${N} > "
  local tmpfile
  tmpfile=$(mktemp)
  curl -i -s -X DELETE "$URL/$1" -H "Authorization: Bearer ${TOKEN}" | tr -d '\r' > "$tmpfile"
  api ${tmpfile}
}

